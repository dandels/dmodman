pub mod download_info;
pub mod download_progress;
mod download_task;
pub mod file_info;
pub mod nxm_url;

pub use self::download_info::*;
pub use self::download_progress::*;
use self::download_task::*;
pub use self::file_info::*;
pub use self::nxm_url::*;
use crate::api::Query;
use crate::api::{ApiError, Client, UpdateStatus};
use crate::cache::{ArchiveEntry, ArchiveFile, ArchiveMetadata, Cache, Cacheable};
use crate::config::{Config, DataPath};
use crate::{util, Logger};
use indexmap::IndexMap;
use std::ffi::OsStr;
use std::io::ErrorKind;
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Downloads {
    pub tasks: Arc<RwLock<IndexMap<u64, DownloadTask>>>,
    pub has_changed: Arc<AtomicBool>,
    logger: Logger,
    cache: Cache,
    client: Client,
    config: Arc<Config>,
    query: Query,
}

impl Downloads {
    pub async fn new(cache: Cache, client: Client, config: Arc<Config>, logger: Logger, query: Query) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(IndexMap::new())),
            has_changed: Arc::new(AtomicBool::new(true)),
            cache,
            client,
            config,
            logger,
            query,
        }
    }

    pub async fn toggle_pause_for(&self, i: usize) {
        let mut lock = self.tasks.write().await;
        let (_, task) = lock.get_index_mut(i).unwrap();
        task.toggle_pause().await;
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn try_queue(&self, nxm_str: &str) {
        let nxm;
        match NxmUrl::from_str(nxm_str) {
            Ok(n) => nxm = n,
            Err(e) => {
                if let ApiError::Expired = e {
                    self.logger.log(format!("nxm url has expired: {nxm_str}"));
                    return;
                } else {
                    self.logger.log(format!("Unable to parse string \"{nxm_str}\" as nxm url: {e}"));
                    return;
                }
            }
        }

        let url = match self.query.download_link(&nxm).await {
            Ok(url) => url,
            Err(_e) => return,
        };
        let file_name = util::file_name_from_url(&url);

        if let Some(task) = self.tasks.write().await.get_mut(&nxm.file_id) {
            match task.dl_info.get_state() {
                DownloadState::Downloading => {
                    self.logger.log(format!("Download of {} is already in progress.", file_name));
                    return;
                }
                DownloadState::Done => {
                    self.logger.log(format!(
                        "{} was recently downloaded but no longer exists. Downloading again...",
                        file_name
                    ));
                    let _ = task.start().await;
                    self.has_changed.store(true, Ordering::Relaxed);
                    return;
                }
                // Restart the download using the new download link.
                _ => {
                    task.dl_info.url = url.clone();
                    if let Err(()) = task.start().await {
                        self.logger.log(format!("Failed to restart download for {}", &file_name));
                    }
                    if let Err(e) = task.dl_info.save(DataPath::DownloadInfo(&self.config, &task.dl_info)).await {
                        self.logger.log(format!("Couldn't store new download url for {}: {}", &file_name, e));
                    }
                    return;
                }
            }
        } // an else {} branch wouldn't drop the lock here, causing self.add() to deadlock
        let f_info = FileInfo::new(nxm.domain_name, nxm.mod_id, nxm.file_id, file_name);
        self.add(DownloadInfo::new(f_info, url)).await;
    }

    pub async fn add(&self, dl_info: DownloadInfo) {
        let mut task = DownloadTask::new(
            self.cache.clone(),
            self.client.clone(),
            self.config.clone(),
            self.logger.clone(),
            dl_info.clone(),
            self.clone(),
            self.query.clone(),
        );

        match dl_info.get_state() {
            DownloadState::Paused => {}
            _ => { let _ = task.start().await; }
        }
        self.tasks.write().await.insert(dl_info.file_info.file_id, task);
        self.has_changed.store(true, Ordering::Relaxed);
    }

    async fn refresh_update_status(&self, fi: &FileInfo) -> UpdateStatus {
        if let Some(file_list) = self.cache.file_lists.get(&fi.game, fi.mod_id).await {
            if file_list.files.is_empty() {
                return UpdateStatus::Invalid(0);
            }
            /* Assume the user has noticed the other files in this mod.
             * For files that aren't out of date, clear the HasNewFile flag and set UpdateStatus to UpToDate(time) where
             * time is the timestamp of the newest file in the mod. */
            let latest_timestamp = file_list.files.last().unwrap().uploaded_timestamp;

            if file_list.file_updates.binary_search_by(|upd| fi.file_id.cmp(&upd.old_file_id)).is_ok() {
                return UpdateStatus::OutOfDate(latest_timestamp);
            }
            if let Some(filedata_heap) = self.cache.metadata_index.get_modfiles(&fi.game, &fi.mod_id).await {
                for fdata in filedata_heap.iter() {
                    // TODO recover from UpdateStatus::Invalid
                    if let UpdateStatus::UpToDate(_) | UpdateStatus::HasNewFile(_) = fdata.update_status.to_enum() {
                        fdata
                            .propagate_update_status(
                                &self.config,
                                &self.logger,
                                &UpdateStatus::UpToDate(latest_timestamp),
                            )
                            .await;
                    } else {
                        // Probably doesn't make sense to do anything in the other cases..?
                    }
                }
            }
            UpdateStatus::UpToDate(latest_timestamp)
        } else {
            self.logger.log("Couldn't check update status for {mod_id}: file list doesn't exist in db.");
            UpdateStatus::Invalid(0)
        }
    }

    async fn update_metadata(&self, fi: &FileInfo) -> Result<(), ApiError> {
        let (game, mod_id) = (&fi.game, fi.mod_id);

        // If the mod info for this file doesn't exist, fetch it from the API
        if let Some(mfd) = self.cache.metadata_index.get_by_file_id(&fi.file_id).await {
            if mfd.mod_info.read().await.is_none() {
                match self.query.mod_info(&mfd.game, mfd.mod_id).await {
                    Ok(_) => self.logger.log(format!("{} was missing its mod info.", fi.file_name)),
                    Err(e) => self.logger.log(format!("Failed to query mod info for {}: {e}", fi.file_name)),
                }
            }
        }

        // same for file list
        if let Some(fl) = self.cache.file_lists.get(game, mod_id).await {
            if fl.files.binary_search_by(|fd| fd.file_id.cmp(&fi.file_id)).is_err() {
                match self.query.file_list(game, mod_id).await {
                    Ok(_) => self.logger.log(format!("{} was missing its file list.", fi.file_name)),
                    Err(e) => self.logger.log(format!("Failed to query file list for {}: {e}", fi.file_name)),
                }
            }
        }
        let update_status = self.refresh_update_status(fi).await;
        let metadata = Arc::new(ArchiveMetadata::new(fi.clone(), update_status));
        let path = self.config.download_dir().join(&fi.file_name);
        // Failing this would mean that the just downloaded file is inaccessible
        if let Ok(archive) =
            ArchiveFile::new(&self.logger, &self.cache.installed, &path, Some(metadata.clone())).await
        {
            if let Err(e) = metadata.save(DataPath::ArchiveMetadata(&self.config, &archive.file_name)).await {
                self.logger.log(format!("Unable to save metadata for {}: {e}", &fi.file_name));
            }
            let entry = ArchiveEntry::File(Arc::new(archive));
            self.cache.archives.add_archive(entry.clone()).await;
        }
        Ok(())
    }

    pub async fn delete(&self, i: usize) {
        let mut tasks_lock = self.tasks.write().await;
        let (_, mut task) = tasks_lock.shift_remove_index(i).unwrap();
        if let DownloadState::Done = task.dl_info.get_state() {
            self.has_changed.store(true, Ordering::Relaxed);
            return;
        }
        task.stop();
        let mut path = self.config.download_dir();
        path.push(format!("{}.part", &task.dl_info.file_info.file_name));
        if fs::remove_file(path.clone()).await.is_err() {
            self.logger.log(format!("Unable to delete {:?}.", &path));
        }
        path.pop();
        path.push(format!("{}.part.json", &task.dl_info.file_info.file_name));
        if fs::remove_file(path.clone()).await.is_err() {
            self.logger.log(format!("Unable to delete {:?}.", &path));
        }
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn resume_on_startup(&self) {
        if let Ok(mut file_stream) = fs::read_dir(&self.config.download_dir()).await {
            while let Some(f) = file_stream.next_entry().await.unwrap() {
                if f.path().is_file() && f.path().extension().and_then(OsStr::to_str) == Some("part") {
                    let part_json_file = f.path().with_file_name(format!("{}.json", f.file_name().to_string_lossy()));
                    match DownloadInfo::load(part_json_file).await {
                        Ok(dl_info) => {
                            self.add(dl_info).await;
                        }
                        Err(ref e) => {
                            if e.kind() == ErrorKind::NotFound {
                                self.logger.log(format!(
                                    "Metadata for partially downloaded file {:?} is missing.\n
                                         The download needs to be restarted through the Nexus.",
                                    f.file_name()
                                ));
                            } else {
                                self.logger.log(format!(
                                    "Unable to deserialize metadata from {:?}:\n
                                        {}",
                                    f.file_name(),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}
