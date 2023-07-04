pub mod download_status;
pub mod nxm_url;
pub use self::download_status::*;
pub use self::nxm_url::*;

use super::{ApiError, Client};
use crate::api::query::{DownloadLink, FileList, Queriable};
use crate::cache::{Cache, LocalFile, UpdateStatus};
use crate::{config::Config, util, Messages};

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};

use indexmap::IndexMap;
use reqwest::header::RANGE;
use reqwest::StatusCode;
use std::collections::HashMap;
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
    task_handles: Arc<RwLock<HashMap<u64, JoinHandle<Result<(), ApiError>>>>>,
    pub has_changed: Arc<AtomicBool>,
    msgs: Messages,
    cache: Cache,
    client: Client,
    config: Config,
}

impl Downloads {
    pub fn new(cache: &Cache, client: &Client, config: &Config, msgs: &Messages) -> Self {
        Self {
            statuses: Arc::new(RwLock::new(IndexMap::new())),
            task_handles: Arc::new(RwLock::new(HashMap::new())),
            has_changed: Arc::new(AtomicBool::new(false)),
            cache: cache.clone(),
            client: client.clone(),
            config: config.clone(),
            msgs: msgs.clone(),
        }
    }

    pub async fn stop(&self, file_id: &u64) {
        let lock = self.task_handles.write();
        lock.await.get(file_id).unwrap().abort();
    }

    pub async fn get_status(&self, file_id: &u64) -> Option<DownloadStatus> {
        self.statuses.read().await.get(file_id).cloned()
    }

    pub async fn get_by_index(&self, i: usize) -> (u64, DownloadStatus) {
        let lock = self.statuses.read().await;
        let (k, v) = lock.get_index(i).unwrap();
        (*k, v.clone())
    }

    async fn add_status(&self, status: DownloadStatus) {
        self.statuses.write().await.insert(status.file_id, status);
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn queue(&self, nxm_str: String) -> Result<(), ApiError> {
        let nxm = NxmUrl::from_str(&nxm_str)?;
        let dls = DownloadLink::request(
            &self.client,
            self.msgs.clone(),
            vec![
                &nxm.domain_name,
                &nxm.mod_id.to_string(),
                &nxm.file_id.to_string(),
                &nxm.query,
            ],
        )
        .await?;
        self.cache.save_download_links(&dls, &nxm.domain_name, &nxm.mod_id, &nxm.file_id).await?;
        /* The API returns multiple locations for Premium users. The first option is by default the Premium-only
         * global CDN, unless the user has selected a preferred download location.
         * For small files the download URL is the same regardless of location choice.
         * Free-tier users only get one location choice.
         * Anyway, we can just pick the first location.
         */
        let location = &dls.locations.first().unwrap();
        let url: Url = Url::parse(&location.URI)?;

        let file_name = util::file_name_from_url(&url);
        let mut path = self.config.download_dir();
        fs::create_dir_all(&path).await?;
        path.push(&file_name);

        if path.exists() {
            self.msgs.push(format!("{} already exists and won't be downloaded again.", file_name)).await;
            return Ok(());
        } else if self.get_status(&nxm.file_id).await.is_some() {
            self.msgs.push(format!("Download of {} is already in progress.", file_name)).await;
            return Ok(());
        }

        self.download_buffered(url, path, nxm.domain_name.clone(), nxm.mod_id, &file_name, nxm.file_id).await?;
        self.update_metadata_for(&nxm, file_name).await?;

        Ok(())
    }

    async fn download_buffered(
        &self,
        url: Url,
        path: PathBuf,
        game: String,
        mod_id: u32,
        file_name: &str,
        file_id: u64,
    ) -> Result<(), ApiError> {
        let me = self.clone();
        let client = self.client.clone();
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

        let mut status = DownloadStatus::new(
            game,
            mod_id,
            file_name.to_string(),
            file_id,
            bytes_read,
            resp.content_length(),
        );
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
        self.task_handles.write().await.insert(file_id, handle);

        Ok(())
    }

    async fn update_metadata_for(&self, nxm: &NxmUrl, file_name: String) -> Result<(), ApiError> {
        let game = &nxm.domain_name;
        let mod_id = nxm.mod_id;
        /* TODO: should we just do an Md5Search instead? It would allows us to validate the file while getting its
         * metadata.
         * However, md5 searching is currently broken: https://github.com/Nexus-Mods/web-issues/issues/1312 */
        let file_list = match self.cache.file_lists.get((game, mod_id)).await {
            Some(fl) => Some(fl),
            None => match FileList::request(&self.client, self.msgs.clone(), vec![game, &mod_id.to_string()]).await {
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
}
