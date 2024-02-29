use super::DownloadState;
use super::{Client, DownloadInfo, DownloadProgress, Downloads};
use crate::cache::{Cache, Cacheable};
use crate::config::{Config, DataType};
use crate::Logger;

use std::fmt::{Debug, Display};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use reqwest::header::RANGE;
use reqwest::{Response, StatusCode};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::{fs, fs::File};
use tokio::{task, task::JoinHandle};
use tokio_stream::StreamExt;

pub struct DownloadTask {
    cache: Cache,
    client: Client,
    config: Config,
    logger: Logger,
    downloads: Downloads,
    join_handle: Option<JoinHandle<()>>,
    pub dl_info: DownloadInfo,
}

impl DownloadTask {
    pub fn new(
        cache: &Cache,
        client: &Client,
        config: &Config,
        logger: &Logger,
        dl_info: DownloadInfo,
        downloads: Downloads,
    ) -> Self {
        Self {
            cache: cache.clone(),
            client: client.clone(),
            config: config.clone(),
            logger: logger.clone(),
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
                let _ = self.start().await;
            }
            // TODO premium users could get a new download link through the API, without having to visit Nexusmods
            DownloadState::Expired => {
                self.dl_info.set_state(DownloadState::Expired);
                self.logger.log(format!(
                    "Download link for {} expired, please download again.",
                    self.dl_info.file_info.file_name
                ));
            }
            DownloadState::Done => return,
        }
        self.save_dl_info().await;
    }

    // helper function to reduce repetition in start()
    async fn log_and_set_error<S: Into<String> + Debug + Display>(&self, msg: S) {
        self.logger.log(msg);
        self.dl_info.set_state(DownloadState::Error);
        self.downloads.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn file_exists(&self) -> bool {
        let file_name = &self.dl_info.file_info.file_name;

        let mut path = self.config.download_dir();
        path.push(file_name);

        if path.exists() {
            if self.cache.file_index.get_by_file_id(&self.dl_info.file_info.file_id).await.is_none() {
                self.logger.log(format!("{} already exists but was missing its metadata.", file_name));
                let _ = self.downloads.update_metadata(&self.dl_info.file_info).await;
            } else {
                self.logger.log(format!("{} already exists and won't be downloaded.", file_name));
            }
            return true;
        }

        false
    }

    pub async fn start(&mut self) -> Result<(), ()> {
        if self.file_exists().await {
            return Err(());
        }

        let mut path = self.config.download_dir();

        if let Err(e) = fs::create_dir_all(&path).await {
            self.log_and_set_error(format!("Error when creating download directory: {}", e)).await;
            return Err(());
        }

        self.dl_info.set_state(DownloadState::Downloading);

        let file_name = self.dl_info.file_info.file_name.clone();
        path.push(&file_name);
        let mut part_path = self.config.download_dir();
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
            return Err(());
        }
        let resp = resp.unwrap();

        let file;
        match self.get_open_opts(&resp, resuming_download, &bytes_read).await {
            Some(open_opts) => match open_opts.open(&part_path).await {
                Ok(f) => file = f,
                Err(e) => {
                    self.log_and_set_error(format!("Unable to open {} for writing: {}", file_name, e)).await;
                    return Err(());
                }
            },
            None => return Err(()),
        }

        let downloads = self.downloads.clone();
        let dl_info = self.dl_info.clone();
        let logger = self.logger.clone();
        let file_name = file_name.clone();
        let handle: JoinHandle<()> = task::spawn(async move {
            // The actual downloading is done here
            if let Err(()) = transfer_data(file, resp, &logger, &downloads, &dl_info).await {
                return;
            }

            if fs::rename(part_path.clone(), path).await.is_err() {
                logger.log(format!("Download of {} complete, but unable to remove .part extension.", file_name));
            }

            part_path.pop();
            part_path.push(format!("{}.part.json", file_name));
            if fs::remove_file(&part_path).await.is_err() {
                logger.log(format!("Unable to remove .part.json file after download is complete: {:?}", part_path));
            }

            dl_info.set_state(DownloadState::Done);
            downloads.has_changed.store(true, Ordering::Relaxed);

            if let Err(e) = downloads.update_metadata(&dl_info.file_info).await {
                logger.log(format!("Unable to update metadata for downloaded file {}: {}", file_name, e));
            }
        });
        self.join_handle = Some(handle);
        Ok(())
    }

    /* Sets OpenOptions depending on whether the download is new (200 OK) or resumed (206 PARTIAL_CONTENT).
     * Updates download progress and
     * */
    async fn get_open_opts(
        &mut self,
        resp: &Response,
        resuming_download: bool,
        bytes_read: &Arc<AtomicU64>,
    ) -> Option<OpenOptions> {
        let file_name = &self.dl_info.file_info.file_name;
        let mut open_opts = OpenOptions::new();
        match resp.error_for_status_ref() {
            Ok(resp) => {
                match resp.status() {
                    StatusCode::OK => {
                        self.dl_info.progress = DownloadProgress::new(bytes_read.clone(), resp.content_length());
                        open_opts.write(true).create(true)
                    }
                    StatusCode::PARTIAL_CONTENT => {
                        if resuming_download {
                            self.dl_info.progress.bytes_read = bytes_read.clone();
                        } else {
                            self.logger.log(
                                "Server unexpectedly responded with 206 PARTIAL CONTENT \
                                           when starting download for {file_name}",
                            );
                            self.dl_info.progress = DownloadProgress::new(bytes_read.clone(), resp.content_length());
                        }
                        open_opts.append(true)
                    }
                    // Running into some other non-error status code shouldn't happen.
                    code => {
                        self.log_and_set_error(format!(
                            "Download for {file_name} got unexpected HTTP response: {code}. Please file a bug report.",
                        ))
                        .await;
                        return None;
                    }
                }
            }
            Err(e) => {
                if resp.status() == StatusCode::GONE {
                    self.dl_info.set_state(DownloadState::Expired);
                    self.downloads.has_changed.store(true, Ordering::Relaxed);
                } else {
                    self.log_and_set_error(format!("Download {file_name} failed with error: {}", e.status().unwrap()))
                        .await;
                }
                return None;
            }
        };
        self.save_dl_info().await;
        Some(open_opts)
    }

    async fn save_dl_info(&self) {
        if let Err(e) = self.dl_info.save(self.config.path_for(DataType::DownloadInfo(&self.dl_info))).await {
            self.logger
                .log(format!("Error when saving download state for {}: {}", self.dl_info.file_info.file_name, e));
        }
    }
}

async fn transfer_data(
    file: File,
    resp: Response,
    logger: &Logger,
    downloads: &Downloads,
    dl_info: &DownloadInfo,
) -> Result<(), ()> {
    let mut bufwriter = BufWriter::new(file);
    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
        match item {
            Ok(bytes) => {
                if let Err(e) = bufwriter.write_all(&bytes).await {
                    logger.log(format!("IO error when writing bytes to disk: {}", e));
                    return Err(());
                }
                dl_info.progress.bytes_read.fetch_add(bytes.len() as u64, Ordering::Relaxed);
                downloads.has_changed.store(true, Ordering::Relaxed);
            }
            Err(e) => {
                logger.log(format!("Error during download: {}", e));
                /* The download could fail for network-related reasons. Flush the data we got so that we can
                 * continue it at some later point. */
                if let Err(e) = bufwriter.flush().await {
                    logger.log(format!("IO error when flushing bytes to disk: {}", e));
                    return Err(());
                }
            }
        }
    }
    if let Err(e) = bufwriter.flush().await {
        logger.log(format!("IO error when flushing bytes to disk: {}", e));
        return Err(());
    }
    Ok(())
}
