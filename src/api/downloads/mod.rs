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

use crate::api::query::{DownloadLink, FileList, Md5Search, Queriable};
use crate::api::{ApiError, Client, UpdateStatus};
use crate::cache::{ArchiveEntry, ArchiveFile, ArchiveMetadata, Cache, Cacheable};
use crate::config::{Config, DataPath};
use crate::{util, Logger};

use std::ffi::OsStr;
use std::io::ErrorKind;
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
    logger: Logger,
    cache: Cache,
    client: Client,
    config: Config,
}

impl Downloads {
    pub async fn new(cache: Cache, client: Client, config: Config, logger: Logger) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(IndexMap::new())),
            has_changed: Arc::new(AtomicBool::new(true)),
            cache,
            client,
            config,
            logger,
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

        let url = match self.request_download_link(&nxm).await {
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
                    if let Err(e) = task.dl_info.save(DataPath::DownloadInfo(&self.config, &task.dl_info).into()).await
                    {
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
        let mut task =
            DownloadTask::new(&self.cache, &self.client, &self.config, &self.logger, dl_info.clone(), self.clone());

        if task.file_exists().await {
            return;
        }

        match dl_info.get_state() {
            DownloadState::Paused => {}
            _ => if let Ok(()) = task.start().await {},
        }
        self.tasks.write().await.insert(dl_info.file_info.file_id, task);
        self.has_changed.store(true, Ordering::Relaxed);
    }

    async fn request_download_link(&self, nxm: &NxmUrl) -> Result<Url, ApiError> {
        match DownloadLink::request(
            &self.client,
            // TODO get rid of passing an array as argument
            &[
                &nxm.domain_name,
                &nxm.mod_id.to_string(),
                &nxm.file_id.to_string(),
                &nxm.query,
            ],
        )
        .await
        {
            Ok(dl_links) => {
                self.cache.save_download_links(&dl_links, &nxm.domain_name, nxm.mod_id, nxm.file_id).await?;
                /* The API returns multiple locations for Premium users. The first option is by default the Premium-only
                 * global CDN, unless the user has selected a preferred download location.
                 * For small files the download URL is the same regardless of location choice.
                 * Free-tier users only get one location choice.
                 * Anyway, we can just pick the first location. */
                let location = dl_links.locations.first().unwrap();
                match Url::parse(&location.URI) {
                    Ok(url) => Ok(url),
                    Err(e) => {
                        self.logger.log(format!(
                            "Failed to parse URI in response from Nexus: {}. \
                                                Please file a bug about this.",
                            &location.URI
                        ));
                        Err(e.into())
                    }
                }
            }
            Err(e) => {
                self.logger.log(format!("Failed to query download links from Nexus: {}", e));
                Err(e)
            }
        }
    }

    async fn update_metadata(&self, fi: &FileInfo) -> Result<(), ApiError> {
        let (game, mod_id) = (&fi.game, fi.mod_id);

        // Try use cached value for this mod, otherwise query API
        let file_list: Option<Arc<FileList>> = 'fl: {
            if let Some(fl) = self.cache.file_lists.get(game, mod_id).await {
                // TODO maybe get the file details here while at it..?
                if fl.files.binary_search_by(|fd| fd.file_id.cmp(&fi.file_id)).is_ok() {
                    break 'fl Some(fl);
                }
            }
            match FileList::request(&self.client, &[game, &mod_id.to_string()]).await {
                Ok(mut fl) => {
                    self.cache.format_file_list(&mut fl, game, mod_id).await;
                    let fl = Arc::new(fl);
                    if let Err(e) = self.cache.save_file_list(fl.clone(), game, mod_id).await {
                        self.logger.log(format!("Unable to save file list for {} mod {}: {}", game, mod_id, e));
                    }
                    Some(fl)
                }
                Err(e) => {
                    self.logger.log(format!("Unable to query file list for {} mod {}: {}", game, mod_id, e));
                    None
                }
            }
        };

        /* Assume the user has noticed the other files in this mod.
         * For files that aren't out of date, clear the HasNewFile flag and set UpdateStatus to UpToDate(time) where
         * time is the timestamp of the newest file in the mod. */
        // TODO if user downloads outdated file it won't be shown as outdated
        let latest_timestamp = file_list.and_then(|fl| fl.files.last().cloned()).unwrap().uploaded_timestamp;
        {
            if let Some(filedata_heap) = self.cache.metadata_index.get_modfiles(game, &mod_id).await {
                for fdata in filedata_heap.iter() {
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
        }

        let metadata = Arc::new(ArchiveMetadata::new(fi.clone(), UpdateStatus::UpToDate(latest_timestamp)));
        let path = self.config.download_dir().join(&fi.file_name);
        // Failing this would mean that the just downloaded file is inaccessible
        if let Some(archive) =
            ArchiveFile::new(&self.logger, &self.cache.installed, &path, Some(metadata.clone())).await
        {
            self.verify_hash(&metadata).await;
            if let Err(e) =
                metadata.save(DataPath::ArchiveMetadata(&self.config, &archive.file_name).into()).await
            {
                self.logger.log(format!("Unable to save metadata for {}: {e}", &fi.file_name));
            }
            let entry = ArchiveEntry::File(Arc::new(archive));
            self.cache.archives.add_archive(entry.clone()).await;
        }
        Ok(())
    }

    async fn verify_hash(&self, metadata: &ArchiveMetadata) {
        let mut path = self.config.download_dir();
        path.push(&metadata.file_name);
        match util::md5sum(path).await {
            Ok(md5) => match Md5Search::request(&self.client, &[&metadata.game, &md5]).await {
                Ok(query_res) => {
                    match query_res.results.iter().find(|fd| fd.file_details.file_id == metadata.file_id) {
                        Some(md5result) => {
                            self.cache.save_md5result(md5result).await;
                            if !(md5.eq(&md5result.file_details.md5)
                                && metadata.file_name.eq(&md5result.file_details.file_name))
                            {
                                self.logger.log(format!(
                                    "Warning: API returned unexpected response when checking hash for {}",
                                    &metadata.file_name
                                ));
                                let mi = &md5result.r#mod;
                                let fd = &md5result.file_details;
                                self.logger.log(format!("Found {:?}: {} ({})", mi.name, fd.name, fd.file_name));
                            }
                        }
                        None => {
                            self.logger.log(format!(
                                "Failed to verify hash for {}. Found this instead:",
                                metadata.file_name
                            ));
                            for res in query_res.results {
                                let mi = &res.r#mod;
                                let fd = &res.file_details;
                                self.logger.log(format!("\t{:?}: {} ({})", mi.name, fd.name, fd.file_name));
                            }
                        }
                    }
                }
                Err(e) => {
                    self.logger.log(format!("Unable to verify integrity of {}: {e}", &metadata.file_name));
                    self.logger.log("This could mean the download got corrupted. See README for details.");
                }
            },
            Err(e) => {
                self.logger.log(format!("Error when checking hash for {}. {e}", metadata.file_name));
            }
        }
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
