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

        self::resume_on_startup(downloads.clone()).await;

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

        if let Some(task) = self.tasks.write().await.get_mut(&nxm.file_id) {
            match task.dl_info.get_state() {
                DownloadState::Downloading => {
                    self.msgs.push(format!("Download of {} is already in progress.", file_name)).await;
                    return;
                }
                //
                DownloadState::Done => {
                    self.msgs
                        .push(format!(
                            "{} was recently downloaded but no longer exists. Downloading again...",
                            file_name
                        ))
                        .await;
                    let _ = task.try_start().await;
                    self.has_changed.store(true, Ordering::Relaxed);
                    return;
                }
                // Restart the download using the new download link.
                _ => {
                    task.dl_info.url = url.clone();
                    if let Err(e) = task.dl_info.save(self.config.path_for(PathType::DownloadInfo(&task.dl_info))).await
                    {
                        self.msgs.push(format!("Couldn't store new download url for {}: {}", &file_name, e)).await;
                    }
                    if let Err(()) = task.try_start().await {
                        self.msgs.push(format!("Failed to restart download for {}", &file_name)).await;
                    }
                }
            }
        };
        let f_info = FileInfo::new(nxm.domain_name, nxm.mod_id, nxm.file_id, file_name);
        self.add(DownloadInfo::new(f_info, url)).await;
    }

    async fn add(&self, dl_info: DownloadInfo) {
        let mut task =
            DownloadTask::new(&self.cache, &self.client, &self.config, &self.msgs, dl_info.clone(), self.clone());
        if task.try_start().await.is_ok() {
            self.tasks.write().await.insert(dl_info.file_info.file_id, task);
            self.has_changed.store(true, Ordering::Relaxed);
        }
    }

    async fn parse_nxm(&self, nxm_str: &str) -> Result<(NxmUrl, Url), ApiError> {
        let nxm = NxmUrl::from_str(nxm_str)?;
        let dls = DownloadLink::request(
            &self.client,
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
                        self.msgs.push(format!("Unable to save file list for {} mod {}: {}", game, mod_id, e)).await;
                    }
                    Some(fl)
                }
                Err(e) => {
                    self.msgs.push(format!("Unable to query file list for {} mod {}: {}", game, mod_id, e)).await;
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
                                self.msgs.push(format!("Couldn't set UpdateStatus for {}: {}", lf.file_name, e)).await;
                            }
                        }
                        // Probably doesn't make sense to do anything in the other cases..?
                        _ => {}
                    }
                }
            }
        }

        let lf = LocalFile::new(fi, UpdateStatus::UpToDate(latest_timestamp));
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
                    if !(md5.eq(&query_res.results.file_details.md5)
                        && local_file.file_name.eq(&query_res.results.file_details.file_name))
                    {
                        self.msgs
                            .push(format!(
                                "Warning: API returned unexpected file when checking hash for {}",
                                &local_file.file_name
                            ))
                            .await;
                        let mi = &query_res.results.r#mod;
                        let fd = &query_res.results.file_details;
                        self.msgs.push(format!("Found {}: {} ({})", mi.name, fd.name, fd.file_name)).await;
                        self.msgs.push("This should be reported as a Nexus bug. See README for details.").await;
                    }
                } else {
                    self.msgs.push(format!("Unable to verify integrity of: {}", &local_file.file_name)).await;
                    self.msgs.push("This could mean the download got corrupted. See README for details.").await;
                }
            }
            Err(e) => {
                self.msgs.push(format!("Error when checking hash for: {}", local_file.file_name)).await;
                self.msgs.push(format!("{}", e)).await;
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
            self.msgs.push(format!("Unable to delete {:?}.", &path)).await;
        }
        path.pop();
        path.push(format!("{}.part.json", &task.dl_info.file_info.file_name));
        if fs::remove_file(path.clone()).await.is_err() {
            self.msgs.push(format!("Unable to delete {:?}.", &path)).await;
        }
        self.has_changed.store(true, Ordering::Relaxed);
    }
}

async fn resume_on_startup(dls: Downloads) {
    if let Ok(mut file_stream) = fs::read_dir(&dls.config.download_dir()).await {
        while let Some(f) = file_stream.next_entry().await.unwrap() {
            if f.path().is_file() && f.path().extension().and_then(OsStr::to_str) == Some("part") {
                let part_json_file = f.path().with_file_name(format!("{}.json", f.file_name().to_string_lossy()));
                if let Ok(dl_info) = DownloadInfo::load(part_json_file).await {
                    let mut task = DownloadTask::new(
                        &dls.cache,
                        &dls.client,
                        &dls.config,
                        &dls.msgs,
                        dl_info.clone(),
                        dls.clone(),
                    );
                    match dl_info.get_state() {
                        DownloadState::Paused => {
                            dls.tasks.write().await.insert(task.dl_info.file_info.file_id, task);
                        }
                        _ => {
                            if task.try_start().await.is_ok() {
                                dls.tasks.write().await.insert(task.dl_info.file_info.file_id, task);
                            }
                        }
                    }
                }
            }
        }
    }
}
