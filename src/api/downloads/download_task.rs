use super::DownloadState;
use super::{ApiError, Client, DownloadInfo, DownloadProgress, Downloads};
use crate::cache::{Cache, Cacheable};
use crate::config::{Config, PathType};
use crate::Messages;

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use reqwest::header::RANGE;
use reqwest::StatusCode;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::{task, task::JoinHandle};
use tokio_stream::StreamExt;

pub struct DownloadTask {
    cache: Cache,
    client: Client,
    config: Config,
    msgs: Messages,
    downloads: Downloads,
    join_handle: Option<JoinHandle<Result<(), ApiError>>>,
    pub dl_info: DownloadInfo,
}

impl DownloadTask {
    pub fn new(
        cache: &Cache,
        client: &Client,
        config: &Config,
        msgs: &Messages,
        dl_info: DownloadInfo,
        downloads: Downloads,
    ) -> Self {
        Self {
            cache: cache.clone(),
            client: client.clone(),
            config: config.clone(),
            msgs: msgs.clone(),
            dl_info,
            downloads,
            join_handle: None,
        }
    }

    pub fn stop(&mut self) {
        if let Some(handle) = &self.join_handle {
            handle.abort();
        }
    }

    pub async fn toggle_pause(&mut self) {
        match self.dl_info.get_state() {
            DownloadState::Downloading => {
                if let Some(handle) = &self.join_handle {
                    handle.abort();
                }
                self.dl_info.set_state(DownloadState::Paused);
            }
            DownloadState::Paused | DownloadState::Error => {
                self.dl_info.set_state(DownloadState::Downloading);
                let _ = self.try_start().await;
            }
            // TODO premium users could get a new download link through the API, without having to visit Nexusmods
            DownloadState::Expired => {
                self.dl_info.set_state(DownloadState::Expired);
                self.msgs
                    .push(format!(
                        "Download link for {} expired, please download again.",
                        self.dl_info.file_info.file_name
                    ))
                    .await;
            }
            DownloadState::Done => {}
        }
        match self.dl_info.save(self.config.path_for(PathType::DownloadInfo(&self.dl_info))).await {
            Ok(()) => {}
            Err(e) => {
                self.msgs
                    .push(format!(
                        "IO error when saving download state for {}: {}",
                        self.dl_info.file_info.file_name, e
                    ))
                    .await;
            }
        }
    }

    // helper function to reduce repetition instart()
    async fn log_and_set_error<S: Into<String>>(&self, msg: S) {
        self.msgs.push(msg).await;
        self.dl_info.set_state(DownloadState::Error);
        self.downloads.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn try_start(&mut self) -> Result<(), ()> {
        let file_name = &self.dl_info.file_info.file_name;

        let mut path = self.config.download_dir();

        match fs::create_dir_all(&path).await {
            Ok(()) => {}
            Err(e) => {
                self.log_and_set_error(format!("Error when creating download directory: {}", e)).await;
                return Err(());
            }
        }
        path.push(file_name);

        if path.exists() {
            if self.cache.file_index.file_id_map.read().await.get(&self.dl_info.file_info.file_id).is_none() {
                self.msgs.push(format!("{} already exists but was missing its metadata.", file_name)).await;
                let _ = self.downloads.update_metadata(self.dl_info.file_info.clone()).await;
            } else {
                self.msgs.push(format!("{} already exists and won't be downloaded.", file_name)).await;
            }
            return Err(());
        }
        self.start(path).await;
        Ok(())
    }

    async fn start(&mut self, path: PathBuf) {
        let file_name = &self.dl_info.file_info.file_name;
        let mut part_path = path.clone();
        part_path.pop();
        part_path.push(format!("{}.part", file_name));

        let mut builder = self.client.build_request(self.dl_info.url.clone()).unwrap();

        /* The HTTP Range header is used to resume downloads.
         * https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Range */
        let bytes_read = Arc::new(AtomicU64::new(0));

        let resuming_download = part_path.exists();
        if resuming_download {
            bytes_read.store(fs::metadata(&part_path).await.unwrap().len(), Ordering::Relaxed);
            builder = builder.header(RANGE, format!("bytes={:?}-", bytes_read));
        }

        let resp = builder.send().await;
        if resp.is_err() {
            self.log_and_set_error("Unable to contact nexus server to start download.").await;
            return;
        }
        let resp = resp.unwrap();

        let mut open_opts = OpenOptions::new();
        #[allow(clippy::needless_late_init)] // false clippy positive methinks
        let open_result;
        match resp.error_for_status_ref() {
            Ok(resp) => {
                open_result = match resp.status() {
                    StatusCode::OK => {
                        self.dl_info.progress = DownloadProgress::new(bytes_read.clone(), resp.content_length());
                        open_opts.write(true).create(true).open(&part_path).await
                    }
                    StatusCode::PARTIAL_CONTENT => {
                        if resuming_download {
                            self.dl_info.progress.bytes_read = bytes_read.clone();
                        } else {
                            self.dl_info.progress = DownloadProgress::new(bytes_read.clone(), resp.content_length());
                        }
                        open_opts.append(true).open(&part_path).await
                    }
                    // Running into some other non-error status code shouldn't happen.
                    code => {
                        self.log_and_set_error(format!(
                            "Download {file_name} got unexpected HTTP response: {code}. Please file a bug report.",
                        ))
                        .await;
                        return;
                    }
                };
            }
            Err(e) => {
                if resp.status() == StatusCode::GONE {
                    self.dl_info.set_state(DownloadState::Expired);
                    self.downloads.has_changed.store(true, Ordering::Relaxed);
                } else {
                    self.log_and_set_error(format!("Download {file_name} failed with error: {}", e.status().unwrap()))
                        .await;
                }
                return;
            }
        }
        if let Err(e) = open_result {
            self.log_and_set_error(format!("Unable to open {file_name} for writing: {}", e)).await;
            return;
        }
        let mut file = open_result.unwrap();

        let downloads = self.downloads.clone();
        let fi = self.dl_info.file_info.clone();
        let dl_info = self.dl_info.clone();
        let msgs = self.msgs.clone();
        let handle: JoinHandle<Result<(), ApiError>> = task::spawn(async move {
            let mut bufwriter = BufWriter::new(&mut file);
            let mut stream = resp.bytes_stream();

            while let Some(item) = stream.next().await {
                match item {
                    Ok(bytes) => {
                        bufwriter.write_all(&bytes).await?;
                        bytes_read.fetch_add(bytes.len() as u64, Ordering::Relaxed);
                        downloads.has_changed.store(true, Ordering::Relaxed);
                    }
                    Err(e) => {
                        /* The download could fail for network-related reasons. Flush the data we got so that we can
                         * continue it at some later point. */
                        bufwriter.flush().await?;
                        return Err(ApiError::from(e));
                    }
                }
            }
            bufwriter.flush().await?;
            if fs::rename(part_path.clone(), path).await.is_err() {
                msgs.push(format!(
                    "Download of {} complete, but unable to remove .part extension.",
                    dl_info.file_info.file_name
                ))
                .await;
            }

            part_path.pop();
            part_path.push(format!("{}.part.json", fi.file_name));
            if fs::remove_file(&part_path).await.is_err() {
                msgs.push(format!("Unable to remove .part.json file after download is complete: {:?}", part_path)).await
            }

            downloads.update_metadata(fi).await?;
            dl_info.set_state(DownloadState::Done);
            downloads.has_changed.store(true, Ordering::Relaxed);
            Ok(())
        });
        self.join_handle = Some(handle);
    }
}
