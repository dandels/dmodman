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

use crate::api::query::{md5_search::*, DownloadLink, FileList, Queriable};
use crate::api::{ApiError, Client};
use crate::cache::{Cache, Cacheable, LocalFile, UpdateStatus};
use crate::config::{Config, PathType};
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
    pub async fn new(cache: &Cache, client: &Client, config: &Config, logger: &Logger) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(IndexMap::new())),
            has_changed: Arc::new(AtomicBool::new(true)),
            cache: cache.clone(),
            client: client.clone(),
            config: config.clone(),
            logger: logger.clone(),
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

        let url;
        match self.request_download_link(&nxm).await {
            Ok(u) => url = u,
            Err(_e) => return,
        }
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
                    if let Err(e) = task.dl_info.save(self.config.path_for(PathType::DownloadInfo(&task.dl_info))).await
                    {
                        self.logger.log(format!("Couldn't store new download url for {}: {}", &file_name, e));
                    }
                    return;
                }
            }
        } // Important to drop the lock here or self.add() deadlocks
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
            // TODO get rid of passing a vec as argument
            vec![
                &nxm.domain_name,
                &nxm.mod_id.to_string(),
                &nxm.file_id.to_string(),
                &nxm.query,
            ],
        )
        .await
        {
            Ok(dl_links) => {
                self.cache.save_download_links(&dl_links, &nxm.domain_name, &nxm.mod_id, &nxm.file_id).await?;
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
        /* TODO: If the FileList isn't found handle this as a foreign file, however they're going to be dealt with.
         * TODO: Should we just do an Md5Search instead? It would allows us to validate the file while getting its
         * metadata.
         * However, md5 searching might still be broken: https://github.com/Nexus-Mods/web-issues/issues/1312 */
        let file_list: Option<FileList> = 'fl: {
            if let Some(fl) = self.cache.file_lists.get((game, mod_id)).await {
                if fl.files.iter().any(|fd| fd.file_id == fi.file_id) {
                    break 'fl Some(fl);
                }
            }
            match FileList::request(&self.client, vec![game, &mod_id.to_string()]).await {
                Ok(fl) => {
                    if let Err(e) = self.cache.save_file_list(&fl, game, mod_id).await {
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

        let latest_timestamp = file_list.and_then(|fl| fl.files.iter().last().cloned()).unwrap().uploaded_timestamp;
        {
            if let Some(filedata_heap) = self.cache.file_index.mod_file_map.read().await.get(&(game.to_owned(), mod_id))
            {
                for fdata in filedata_heap.iter() {
                    let mut lf = fdata.local_file.write().await;
                    match lf.update_status {
                        UpdateStatus::UpToDate(_) | UpdateStatus::HasNewFile(_) => {
                            lf.update_status = UpdateStatus::UpToDate(latest_timestamp);
                            let path = self.config.path_for(PathType::LocalFile(&lf));
                            if let Err(e) = lf.save(path).await {
                                self.logger.log(format!("Couldn't set UpdateStatus for {}: {}", lf.file_name, e));
                            }
                        }
                        // Probably doesn't make sense to do anything in the other cases..?
                        _ => {}
                    }
                }
            }
        }

        let lf = LocalFile::new(fi.clone(), UpdateStatus::UpToDate(latest_timestamp));
        self.verify_hash(&lf).await;
        self.cache.save_local_file(lf.clone()).await?;
        Ok(())
    }

    async fn verify_hash(&self, local_file: &LocalFile) {
        let mut path = self.config.download_dir();
        path.push(&local_file.file_name);
        match util::md5sum(path).await {
            Ok(md5) => {
                if let Ok(query_res) = Md5Search::request(&self.client, vec![&local_file.game, &md5]).await {
                    // Uncomment to save API response
                    //let _ = query_res
                    //    .save(self.config.path_for(PathType::Md5Search(
                    //        &local_file.game,
                    //        &local_file.mod_id,
                    //        &local_file.file_id,
                    //    )))
                    //    .await;

                    if let Some(md5result) =
                        query_res.results.iter().find(|fd| fd.file_details.file_id == local_file.file_id)
                    {
                        if !(md5.eq(&md5result.file_details.md5)
                            && local_file.file_name.eq(&md5result.file_details.file_name))
                        {
                            self.logger.log(format!(
                                "Warning: API returned unexpected file when checking hash for {}",
                                &local_file.file_name
                            ));
                            let mi = &md5result.r#mod;
                            let fd = &md5result.file_details;
                            self.logger.log(format!("Found {:?}: {} ({})", mi.name, fd.name, fd.file_name));
                            self.logger.log("This should be reported as a Nexus bug. See README for details.");
                        }
                        // Early return if success, else fall through to error reporting.
                        return;
                    }
                }
                self.logger.log(format!("Unable to verify integrity of: {}", &local_file.file_name));
                self.logger.log("This could mean the download got corrupted. See README for details.");
            }
            Err(e) => {
                self.logger.log(format!("Error when checking hash for: {}", local_file.file_name));
                self.logger.log(format!("{}", e));
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
                // Resume incomplete downloads
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
