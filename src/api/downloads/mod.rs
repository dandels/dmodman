pub mod download_info;
pub mod download_progress;
pub mod download_state;
mod download_task;
pub mod file_info;
pub mod nxm_url;

pub use self::download_info::*;
pub use self::download_progress::*;
pub use self::download_state::*;
use self::download_task::*;
pub use self::file_info::*;
pub use self::nxm_url::*;

use crate::api::query::{DownloadLink, FileList, Queriable};
use crate::api::{ApiError, Client};
use crate::cache::{Cache, Cacheable, LocalFile, UpdateStatus};
use crate::config::{Config, PathType};
use crate::{util, Messages};

use std::ffi::OsStr;
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use indexmap::IndexMap;
use tokio::fs;
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
    pub async fn new(cache: &Cache, client: &Client, config: &Config, msgs: &Messages) -> Self {
        let downloads = Self {
            tasks: Arc::new(RwLock::new(IndexMap::new())),
            has_changed: Arc::new(AtomicBool::new(true)),
            cache: cache.clone(),
            client: client.clone(),
            config: config.clone(),
            msgs: msgs.clone(),
        };

        self::resume_on_startup(config, downloads.clone()).await;

        downloads
    }

    pub async fn toggle_pause_for(&self, i: usize) {
        let mut lock = self.tasks.write().await;
        let (_, task) = lock.get_index_mut(i).unwrap();
        task.toggle_pause().await;
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn queue(&self, nxm_str: String) {
        let res = self.parse_nxm(&nxm_str).await;
        if let Err(e) = res {
            self.msgs.push(format!("Unable to parse nxm string \"{}\" {}", nxm_str, e)).await;
            return;
        }
        let (nxm, url) = res.unwrap();
        let file_name = util::file_name_from_url(&url);

        if let Some(dl) = self.tasks.read().await.get(&nxm.file_id) {
            match download_state::to_enum(dl.dl_info.state.clone()) {
                DownloadState::Downloading => {
                    self.msgs.push(format!("Download of {} is already in progress.", file_name)).await;
                    return;
                }
                /* Do nothing in the rest of the cases.
                 * The download will be unpaused when the DownloadTask is recreated. */
                DownloadState::Complete => {
                    self.msgs
                        .push(format!(
                            "{} was recently downloaded but no longer exists. Downloading again...",
                            file_name
                        ))
                        .await;
                }
                _ => {}
            }
        }

        // We don't save the LocalFile until after the download, but it's convenient for passing data around.
        let f_info = FileInfo::new(nxm.domain_name, nxm.mod_id, nxm.file_id, file_name);
        self.add(DownloadInfo::new(f_info, url)).await;
    }

    async fn add(&self, dl_info: DownloadInfo) {
        let mut task = DownloadTask::new(&self.client, &self.config, &self.msgs, dl_info.clone(), self.clone());
        {
            task.start().await;
            if let Err(e) = task.dl_info.save(self.config.path_for(PathType::DownloadInfo(&dl_info))).await {
                self.msgs.push(format!("Error when saving download state: {}", e)).await;
            }
        }
        self.tasks.write().await.insert(dl_info.file_info.file_id, task);
        self.has_changed.store(true, Ordering::Relaxed);
    }

    async fn parse_nxm(&self, nxm_str: &str) -> Result<(NxmUrl, Url), ApiError> {
        let nxm = NxmUrl::from_str(nxm_str)?;
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

    async fn update_metadata(&self, fi: FileInfo) -> Result<(), ApiError> {
        let (game, mod_id) = (&fi.game, fi.mod_id);
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
            file_list.and_then(|fl| fl.files.iter().find(|fd| fd.file_id == fi.file_id).cloned()).unwrap();

        // TODO set UpdateStatus for other files in the mod
        let lf = LocalFile::new(fi, UpdateStatus::UpToDate(file_details.uploaded_timestamp));
        self.cache.save_local_file(lf).await?;
        Ok(())
    }
}

async fn resume_on_startup(config: &Config, dls: Downloads) {
    if let Ok(mut file_stream) = fs::read_dir(config.download_dir()).await {
        while let Some(f) = file_stream.next_entry().await.unwrap() {
            if f.path().is_file() && f.path().extension().and_then(OsStr::to_str) == Some("part") {
                let part_json_file = f.path().with_file_name(format!("{}.json", f.file_name().to_string_lossy()));
                if part_json_file.exists() {
                    if let Ok(dl_info) = DownloadInfo::load(part_json_file).await {
                        let mut task = DownloadTask::new(&dls.client, config, &dls.msgs, dl_info.clone(), dls.clone());
                        match download_state::to_enum(dl_info.state.clone()) {
                            DownloadState::Paused => {}
                            _ => {
                                task.start().await;
                            }
                        }
                        dls.tasks.write().await.insert(task.dl_info.file_info.file_id, task);
                    }
                }
            }
        }
    }
}
