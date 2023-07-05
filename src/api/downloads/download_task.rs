use super::{ApiError, Client, DownloadProgress, Downloads};
use crate::cache::LocalFile;
use crate::{config::Config, Messages};
use reqwest::header::RANGE;
use reqwest::StatusCode;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::RwLock;
use tokio::{task, task::JoinHandle};
use tokio_stream::StreamExt;
use url::Url;

pub enum DownloadState {
    Complete,
    Downloading,
    Paused,
}

pub struct DownloadTask {
    client: Client,
    config: Config,
    msgs: Messages,
    url: Url,
    downloads: Downloads,
    join_handle: Option<JoinHandle<Result<(), ApiError>>>,
    pub lf: LocalFile,
    pub state: Arc<RwLock<DownloadState>>,
    pub progress: DownloadProgress,
}

impl DownloadTask {
    pub fn new(
        client: &Client,
        config: &Config,
        msgs: &Messages,
        lf: LocalFile,
        url: Url,
        downloads: Downloads,
    ) -> Self {
        Self {
            client: client.clone(),
            config: config.clone(),
            msgs: msgs.clone(),
            lf,
            url,
            downloads,
            join_handle: None,
            state: Arc::new(RwLock::new(DownloadState::Downloading)),
            progress: DownloadProgress::default(),
        }
    }

    pub async fn toggle_pause(&mut self) {
        let mut resume = false;
        // scope to drop the lock so we can borrow &mut self again
        {
            let mut lock = self.state.write().await;
            match *lock {
                DownloadState::Downloading => {
                    self.join_handle.as_mut().unwrap().abort();
                    *lock = DownloadState::Paused;
                }
                DownloadState::Paused => {
                    resume = true;
                    *lock = DownloadState::Downloading;
                }
                DownloadState::Complete => {}
            }
        }
        if resume {
            // TODO error handling regardless of how download is started
            self.start().await.unwrap()
        }
    }

    pub async fn start(&mut self) -> Result<(), ApiError> {
        let mut path = self.config.download_dir();
        fs::create_dir_all(&path).await?;
        path.push(&self.lf.file_name);

        if path.exists() {
            self.msgs.push(format!("{} already exists and won't be downloaded.", self.lf.file_name)).await;
            return Ok(());
        }

        self.msgs.push(format!("Downloading to {:?}.", path)).await;
        let mut part_path = path.clone();
        part_path.pop();
        part_path.push(format!("{}.part", self.lf.file_name));

        let mut builder = self.client.build_request(self.url.clone())?;

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
                        self.lf.file_name, code
                    ),
                };
            }
            Err(e) => {
                self.msgs
                    .push(format!("Download {} failed with error: {}", self.lf.file_name, e.status().unwrap()))
                    .await;
                return Err(ApiError::from(e));
            }
        }

        self.progress = DownloadProgress::new(bytes_read.clone(), resp.content_length());

        let downloads = self.downloads.clone();
        let state = self.state.clone();
        let lf = self.lf.clone();
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
            downloads.update_metadata(lf).await?;

            let mut lock = state.write().await;
            *lock = DownloadState::Complete;

            Ok(())
        });
        self.join_handle = Some(handle);
        Ok(())
    }
}
