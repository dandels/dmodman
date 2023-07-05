pub mod download_progress;
mod download_task;
pub mod nxm_url;
pub use self::download_progress::*;
use self::download_task::*;
pub use self::nxm_url::*;

use super::{ApiError, Client};
use crate::api::query::{DownloadLink, FileList, Queriable};
use crate::cache::{Cache, LocalFile, UpdateStatus};
use crate::{config::Config, util, Messages};
use indexmap::IndexMap;
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::RwLock;
use url::Url;

#[derive(Clone)]
pub struct Downloads {
    pub tasks: Arc<RwLock<IndexMap<u64, DownloadTask>>>,
    pub has_changed: Arc<AtomicBool>,
    msgs: Messages,
    cache: Cache,
    client: Client,
    config: Config,
}

impl Downloads {
    pub fn new(cache: &Cache, client: &Client, config: &Config, msgs: &Messages) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(IndexMap::new())),
            has_changed: Arc::new(AtomicBool::new(false)),
            cache: cache.clone(),
            client: client.clone(),
            config: config.clone(),
            msgs: msgs.clone(),
        }
    }

    pub async fn toggle_pause_for(&self, i: usize) {
        let mut lock = self.tasks.write().await;
        let (_, task) = lock.get_index_mut(i).unwrap();
        task.toggle_pause().await;
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn queue(&self, nxm_str: String) -> Result<(), ApiError> {
        let (nxm, url) = self.parse_nxm(nxm_str).await?;
        let file_name = util::file_name_from_url(&url);

        if let Some(dl) = self.tasks.read().await.get(&nxm.file_id) {
            match *dl.state.read().await {
                DownloadState::Downloading => {
                    self.msgs.push(format!("Download of {} is already in progress.", file_name)).await;
                    return Ok(());
                }
                // Do nothing in these two cases, the download will be unpaused when the DownloadTask is recreated.
                DownloadState::Complete => {
                    self.msgs
                        .push(format!(
                            "{} was recently downloaded but no longer exists. Downloading again...",
                            file_name
                        ))
                        .await;
                }
                DownloadState::Paused => {}
            }
        }

        // We don't save the LocalFile until after the download, but it's convenient for passing data around.
        let lf = LocalFile::new(&nxm, file_name.clone(), UpdateStatus::OutOfDate(0));
        self.download_modfile(url, lf).await
    }

    async fn download_modfile(&self, url: Url, lf: LocalFile) -> Result<(), ApiError> {
        let mut task = DownloadTask::new(&self.client, &self.config, &self.msgs, lf.clone(), url, self.clone());
        task.start().await?;
        self.tasks.write().await.insert(lf.file_id, task);
        self.has_changed.store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn parse_nxm(&self, nxm_str: String) -> Result<(NxmUrl, Url), ApiError> {
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
         * Anyway, we can just pick the first location. */
        let location = dls.locations.first().unwrap();
        Ok((nxm, Url::parse(&location.URI)?))
    }

    async fn update_metadata(&self, lf: LocalFile) -> Result<(), ApiError> {
        let (game, mod_id) = (&lf.game, lf.mod_id);
        /* TODO: If the FileList isn't found handle this as a foreign file, however they're going to be dealt with.
         * The unwrap() here should be done away with.
         * TODO: Should we just do an Md5Search instead? It would allows us to validate the file while getting its
         * metadata. However, md5 searching is currently broken: https://github.com/Nexus-Mods/web-issues/issues/1312 */
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

        let file_details =
            file_list.and_then(|fl| fl.files.iter().find(|fd| fd.file_id == lf.file_id).cloned()).unwrap();

        // TODO set UpdateStatus for other files in the mod
        let mut lf = lf.clone();
        lf.update_status = UpdateStatus::UpToDate(file_details.uploaded_timestamp);
        self.cache.add_local_file(lf).await?;
        Ok(())
    }
}
