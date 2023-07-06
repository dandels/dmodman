use super::download_state;
use super::download_state::*;
use super::{ApiError, Client, DownloadProgress, Downloads, FileInfo};
use crate::{config::Config, Messages};

use std::sync::{
    atomic::{AtomicU64, AtomicU8, Ordering},
    Arc,
};

use reqwest::header::RANGE;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::{task, task::JoinHandle};
use tokio_stream::StreamExt;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadInfo {
    pub file_info: FileInfo,
    pub url: Url,
    pub state: Arc<AtomicU8>,
}

impl DownloadInfo {
    pub fn new(file_info: FileInfo, url: Url) -> Self {
        Self {
            file_info,
            url,
            state: Arc::new(DL_STATE_DOWNLOADING.into()),
        }
    }
}

pub struct DownloadTask {
    client: Client,
    config: Config,
    msgs: Messages,
    downloads: Downloads,
    join_handle: Option<JoinHandle<Result<(), ApiError>>>,
    pub dl_info: DownloadInfo,
    pub progress: DownloadProgress,
}

impl DownloadTask {
    pub fn new(client: &Client, config: &Config, msgs: &Messages, dl_info: DownloadInfo, downloads: Downloads) -> Self {
        Self {
            client: client.clone(),
            config: config.clone(),
            msgs: msgs.clone(),
            dl_info,
            downloads,
            join_handle: None,
            progress: DownloadProgress::default(),
        }
    }

    pub async fn toggle_pause(&mut self) {
        match download_state::to_enum(self.dl_info.state.clone()) {
            DownloadState::Downloading => {
                self.join_handle.as_mut().unwrap().abort();
                self.dl_info.state.store(DL_STATE_PAUSED, Ordering::Relaxed);
            }
            DownloadState::Paused | DownloadState::Error => {
                self.dl_info.state.store(DL_STATE_DOWNLOADING, Ordering::Relaxed);
                // TODO error handling regardless of how download is started
                self.start().await.unwrap()
            }
            DownloadState::Complete => {}
        }
    }

    pub async fn start(&mut self) -> Result<(), ApiError> {
        let file_name = &self.dl_info.file_info.file_name;

        let mut path = self.config.download_dir();
        fs::create_dir_all(&path).await?;
        path.push(&self.dl_info.file_info.file_name);

        if path.exists() {
            self.msgs.push(format!("{} already exists and won't be downloaded.", file_name)).await;
            return Ok(());
        }

        self.msgs.push(format!("Downloading to {:?}.", path)).await;
        let mut part_path = path.clone();
        part_path.pop();
        part_path.push(format!("{}.part", file_name));

        let mut builder = self.client.build_request(self.dl_info.url.clone())?;

        /* The HTTP Range header is used to resume downloads.
         * https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Range */
        let bytes_read = Arc::new(AtomicU64::new(0));
        if part_path.exists() {
            bytes_read.store(std::fs::metadata(&part_path)?.len(), Ordering::Relaxed);
            builder = builder.header(RANGE, format!("bytes={:?}-", bytes_read));
        }

        let resp = builder.send().await?;

        let mut open_opts = OpenOptions::new();
        let mut file;
        match resp.error_for_status_ref() {
            Ok(resp) => {
                file = match resp.status() {
                    StatusCode::OK => open_opts.write(true).create(true).open(&part_path).await?,
                    StatusCode::PARTIAL_CONTENT => open_opts.append(true).open(&part_path).await?,
                    // Running into some other non-error status code shouldn't happen.
                    code => panic!(
                        "Download {} got unexpected HTTP response: {}. Please file a bug report.",
                        file_name, code
                    ),
                };
            }
            Err(e) => {
                self.msgs.push(format!("Download {} failed with error: {}", file_name, e.status().unwrap())).await;
                return Err(ApiError::from(e));
            }
        }

        self.progress = DownloadProgress::new(bytes_read.clone(), resp.content_length());

        let downloads = self.downloads.clone();
        //let state = self.state.clone();
        let fi = self.dl_info.file_info.clone();
        let state = self.dl_info.state.clone();
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
            std::fs::rename(part_path, path)?;
            downloads.update_metadata(fi).await?;

            state.store(DL_STATE_COMPLETE, Ordering::Relaxed);
            Ok(())
        });
        self.join_handle = Some(handle);
        Ok(())
    }
}
