pub mod download_status;
pub mod nxm_url;
pub use self::download_status::*;
pub use self::nxm_url::*;

use super::{ApiError, Client};
use crate::api::query::{FileList, Queriable};
use crate::cache::{Cache, LocalFile, UpdateStatus};
use crate::{config::Config, util, Messages};

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};

use indexmap::IndexMap;
use reqwest::header::RANGE;
use reqwest::StatusCode;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::RwLock;
use tokio::{task, task::JoinHandle};
use tokio_stream::StreamExt;
use url::Url;

#[derive(Clone)]
pub struct Downloads {
    pub statuses: Arc<RwLock<IndexMap<u64, DownloadStatus>>>,
    pub has_changed: Arc<AtomicBool>,
    msgs: Messages,
    cache: Cache,
    config: Config,
}

impl Downloads {
    pub fn new(cache: &Cache, config: &Config, msgs: &Messages) -> Self {
        Self {
            statuses: Arc::new(RwLock::new(IndexMap::new())),
            has_changed: Arc::new(AtomicBool::new(false)),
            cache: cache.clone(),
            config: config.clone(),
            msgs: msgs.clone(),
        }
    }

    pub async fn get_status(&self, file_id: &u64) -> Option<DownloadStatus> {
        self.statuses.read().await.get(file_id).cloned()
    }

    pub async fn add_status(&self, status: DownloadStatus) {
        self.statuses.write().await.insert(status.file_id, status);
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn add(&self, client: &Client, nxm: &NxmUrl, url: Url) -> Result<(), ApiError> {
        let me = self.clone();
        let file_name = util::file_name_from_url(&url);
        let mut path = me.config.download_dir();
        fs::create_dir_all(&path).await?;
        path.push(&file_name);

        if path.exists() {
            self.msgs.push(format!("{} already exists and won't be downloaded again.", file_name)).await;
            return Ok(());
        } else if me.get_status(&nxm.file_id).await.is_some() {
            self.msgs.push(format!("Download of {} is already in progress.", file_name)).await;
            return Ok(());
        }

        me.download_buffered(client, url, path, &file_name, nxm.file_id).await?;
        me.update_metadata_for(client, nxm, file_name).await?;

        Ok(())
    }

    async fn update_metadata_for(&self, client: &Client, nxm: &NxmUrl, file_name: String) -> Result<(), ApiError> {
        let game = &nxm.domain_name;
        let mod_id = nxm.mod_id;
        /* TODO: should we just do an Md5Search instead? It would allows us to validate the file while getting its
         * metadata.
         * However, md5 searching is currently broken: https://github.com/Nexus-Mods/web-issues/issues/1312 */
        let file_list = match self.cache.file_lists.get((game, mod_id)).await {
            Some(fl) => Some(fl),
            None => match FileList::request(client, self.msgs.clone(), vec![game, &mod_id.to_string()]).await {
                Ok(fl) => {
                    if let Err(e) = self.cache.save_file_list(&fl, game, mod_id).await {
                        self.msgs.push(format!("Unable to save file list for {} mod {}: {}", game, mod_id, e)).await;
                    }
                    Some(fl)
                }
                Err(e) => {
                    self.msgs.push(format!("Unable to query file list for {} mod {}: {}", game, mod_id, e)).await;
                    None
                }
            },
        };

        /* TODO if the FileDetails isn't found handle this as a foreign file, however they're going to be dealt with.
         * The unwrap() here should be done away with. */
        let file_details =
            file_list.and_then(|fl| fl.files.iter().find(|fd| fd.file_id == nxm.file_id).cloned()).unwrap();

        let lf = LocalFile::new(nxm, file_name, UpdateStatus::UpToDate(file_details.uploaded_timestamp));

        self.cache.add_local_file(lf).await?;
        Ok(())
    }

    async fn download_buffered(
        &self,
        client: &Client,
        url: Url,
        path: PathBuf,
        file_name: &str,
        file_id: u64,
    ) -> Result<(), ApiError> {
        let me = self.clone();
        let client = client.clone();
        me.msgs.push(format!("Downloading to {:?}.", path)).await;
        let mut part_path = path.clone();
        part_path.pop();
        part_path.push(format!("{}.part", file_name));

        let mut builder = client.build_request(url)?;

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
                    code => panic!("Download {} got unexpected HTTP response: {}", file_name, code),
                };
            }
            Err(e) => {
                me.msgs
                    .push(format!(
                        "Download {} failed with error: {}",
                        file_name,
                        e.status().unwrap()
                    ))
                    .await;
                return Err(ApiError::from(e));
            }
        }

        let mut status = DownloadStatus::new(file_name.to_string(), file_id, bytes_read, resp.content_length());
        me.add_status(status.clone()).await;

        let handle: JoinHandle<Result<(), ApiError>> = task::spawn(async move {
            let mut bufwriter = BufWriter::new(&mut file);
            let mut stream = resp.bytes_stream();

            while let Some(item) = stream.next().await {
                match item {
                    Ok(bytes) => {
                        bufwriter.write_all(&bytes).await?;
                        status.update_progress(bytes.len() as u64);
                        me.has_changed.store(true, Ordering::Relaxed);
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
            Ok(())
        });

        Ok(())
    }
}
