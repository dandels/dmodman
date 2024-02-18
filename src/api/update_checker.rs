use super::ApiError;
use super::{Client, FileList, FileUpdate, Queriable};
use crate::cache::{Cache, Cacheable, FileData, UpdateStatus};
use crate::config::PathType;
use crate::Config;
use crate::Logger;

use std::collections::BinaryHeap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use tokio::task;

#[derive(Clone)]
pub struct UpdateChecker {
    cache: Cache,
    client: Client,
    config: Config,
    logger: Logger,
}

impl UpdateChecker {
    pub fn new(cache: Cache, client: Client, config: Config, logger: Logger) -> Self {
        Self {
            cache,
            client,
            config,
            logger,
        }
    }

    pub async fn ignore_file(&self, i: usize) {
        let f_lock = self.cache.file_index.files_sorted.read().await;
        let fd = f_lock.get(i).unwrap();
        let mut lf_lock = fd.local_file.write().await;
        if let Some(latest_remote_file) =
            self.cache.file_lists.get((&lf_lock.game, lf_lock.mod_id)).await.unwrap().file_updates.peek()
        {
            lf_lock.update_status = UpdateStatus::IgnoredUntil(latest_remote_file.uploaded_timestamp);

            if let Err(e) = lf_lock.save(self.config.path_for(PathType::LocalFile(&lf_lock))).await {
                self.logger.log(format!("Unable save ignore status for: {e}."));
            }
            self.cache.file_index.has_changed.store(true, Ordering::Relaxed);
        }
    }

    pub async fn update_all(&self) {
        let mods;
        {
            let lock = self.cache.file_index.mod_file_map.read().await;
            mods = lock.clone().into_keys();
        }
        for (game, mod_id) in mods {
            self.update_mod(game, mod_id).await;
        }
        self.logger.log("Finished checking updates.");
    }

    pub async fn update_mod(&self, game: String, mod_id: u32) {
        let me = self.clone();
        task::spawn(async move {
            let lock = me.cache.file_index.mod_file_map.read().await;
            let files = lock.get(&(game.to_owned(), mod_id)).unwrap();

            let mut needs_refresh = false;
            let mut checked: Vec<(Arc<FileData>, UpdateStatus)> = vec![];
            /* First try to check updates with cached values.
             * If the UpdateStatus is already OutOfDate or HasNewFile, there's no reason to query the API.
             * Only query the API if a file is still reported as UpToDate.
             */
            if let Some(fl) = me.cache.file_lists.get((&game, mod_id)).await {
                checked = me.check_mod(files, &fl).await;
                for (_fdata, status) in &checked {
                    if let UpdateStatus::UpToDate(_) = status {
                        needs_refresh = true;
                    }
                }
            } else {
                me.logger.log(format!("Strange, no file list in cache for {mod_id}. Fetching."));
                needs_refresh = true;
            }
            if needs_refresh {
                /* We only need to make one API request per mod, since the response contains info about all files in
                 * that mod. */
                match me.refresh_filelist(&game, mod_id).await {
                    Ok(fl) => {
                        checked = me.check_mod(files, &fl).await;
                    }
                    Err(e) => {
                        me.logger.log(format!("Error when refresh filelist for {mod_id}: {}", e));
                    }
                }
            }
            for (file, new_status) in checked {
                let mut lf = file.local_file.write().await;
                if lf.update_status != new_status {
                    me.logger.log(format!("Setting {} status to {:?}", file.file_details.name, new_status));
                    lf.update_status = new_status;
                    lf.save(me.config.path_for(PathType::LocalFile(&lf))).await.unwrap();
                }
            }
            me.cache.file_index.has_changed.store(true, Ordering::Relaxed);
        });
    }

    async fn refresh_filelist(&self, game: &str, mod_id: u32) -> Result<FileList, ApiError> {
        let file_list = FileList::request(&self.client, &[game, &mod_id.to_string()]).await?;
        self.cache.save_file_list(&file_list, game, mod_id).await?;
        Ok(file_list)
    }

    /* This is complicated and maybe buggy.
     *
     * There are several ways in which a mod can have updates.
     * 1) The FileList response for a mod contains a file_updates array, with which we can iterate over the update chain
     *    for a specific file id. The updates need to be sorted by timestamp or the time complexity of this is O(n²) per
     *    file.
     *    However, The NexusMods community manager, who has been very helpful, couldn't guarantee that the API keeps them
     *    sorted.
     *    To reduce the amount of time spent iterating over file lists, the file updates are put into a binary heap with
     *    a custom ordering based on timestamp, which is done immediately in the deserialization stage.
     *    (Testing suggests that Serde seems to understand BinaryHeap, and deserializes the JSON into heap order rather than
     *    messing up the sorting. The documentation on this was lacking, though.)
     *
     *    The file list for the mod is kept in another binary heap, also based on timestamp.
     *    We then iterate backwards over both lists at once by calling pop()/peek(). This allows us to skip iterating over
     *    any update that is older than our files. The popped updates are then kept in a list, which contains only the
     *    updates newer than the currently inspected file.
     *
     * 2) The category of the file might have changed to OLD_VERSION or ARCHIVED without the update list containing new
     *    versions of that file.
     *
     * 3) Even when neither of these are true, there might be some other new file in the mod. This could be an optional
     *    file, a patch for the main mod or between the mod and some other mod, a new version that doesn't fit in the
     *    previous two categories, etc. Since figuring out these cases is infeasible, we set timestamps on each file's
     *    UpdateStatus (setting it on the latest one isn't enough, as the user could delete it).
     *    If none of the other update conditions are true, we set the file's update status to either HasNewFile or
     *    UpToDate, depending on the timestamp. */
    async fn check_mod(
        &self,
        to_check: &BinaryHeap<Arc<FileData>>,
        file_list: &FileList,
    ) -> Vec<(Arc<FileData>, UpdateStatus)> {
        if to_check.peek().is_none() {
            self.logger.log("Tried to check updates for nonexistent files. This shouldn't happen.");
            return vec![];
        }

        let mut files = to_check.clone();
        let mut updates = file_list.file_updates.clone();
        let mut checked: Vec<(Arc<FileData>, UpdateStatus)> = vec![];
        let latest_local_time = { to_check.peek().unwrap().local_file.write().await.update_status.time() };
        // Here we assume that the last file in the file list is actually the latest, which is probably true.
        let latest_remote_time = file_list.files.last().unwrap().uploaded_timestamp;

        let mut newer_files: Vec<FileUpdate> = vec![];
        while let Some(file) = files.pop() {
            let local_file = file.local_file.read().await;

            match local_file.update_status {
                // No need to check files that are already known to have updates
                UpdateStatus::OutOfDate(_) | UpdateStatus::HasNewFile(_) => {
                    checked.push((file.clone(), local_file.update_status.clone()));
                    continue;
                }
                _ => {}
            }

            let mut has_update = false;

            // enums used by the API
            const OLD_VERSION: u32 = 4;
            const ARCHIVED: u32 = 7;
            if file.file_details.category_id == OLD_VERSION || file.file_details.category_id == ARCHIVED {
                has_update = true;
            } else {
                /* For each file we're checking, we're only concerned about files that are newer than it.
                 * Files that we iterate on after this one can reuse this same information, since both heaps are sorted by
                 * timestamp. */
                while let Some(upd) = updates.peek() {
                    /* The timestamp in the file updates might be slightly later than the one in the FileList, so we
                     * also need to compare file_id's. */
                    if file.file_details.uploaded_timestamp < upd.uploaded_timestamp && file.file_id != upd.new_file_id
                    {
                        newer_files.push(updates.pop().unwrap());
                    } else {
                        break;
                    }
                }
            }

            // the files get popped in descending chronological order, so we need to iterate this in reverse
            for upd in newer_files.iter().rev() {
                if file.file_id == upd.old_file_id {
                    has_update = true;
                    break;
                }
            }
            if has_update {
                match local_file.update_status {
                    // Set file out of date unless this update is ignored
                    UpdateStatus::IgnoredUntil(t) => {
                        if t < latest_remote_time {
                            checked.push((file.clone(), UpdateStatus::OutOfDate(latest_remote_time)));
                        // this is still ignored and we don't touch it
                        } else {
                            checked.push((file.clone(), UpdateStatus::IgnoredUntil(t)));
                        }
                    }
                    _ => {
                        checked.push((file.clone(), UpdateStatus::OutOfDate(latest_remote_time)));
                    }
                }
            // No direct update in update chain, but there might be new files
            } else if latest_local_time < latest_remote_time {
                match local_file.update_status {
                    UpdateStatus::IgnoredUntil(t) => {
                        // another remote file has appeared since updates were ignored
                        if t < latest_remote_time {
                            checked.push((file.clone(), UpdateStatus::HasNewFile(latest_local_time)));
                        // this is still ignored and we don't touch it
                        } else {
                            checked.push((file.clone(), UpdateStatus::IgnoredUntil(t)));
                        }
                    }
                    _ => {
                        checked.push((file.clone(), UpdateStatus::HasNewFile(latest_local_time)));
                    }
                }
            } else {
                checked.push((file.clone(), UpdateStatus::UpToDate(latest_local_time)));
            }
        }
        checked
    }
}

#[cfg(test)]
mod tests {
    use crate::api::{ApiError, Client, UpdateChecker};
    use crate::cache::Cache;
    use crate::cache::UpdateStatus;
    use crate::ConfigBuilder;
    use crate::Logger;

    #[tokio::test]
    async fn block_test_request() -> Result<(), ApiError> {
        let game = "morrowind";
        let mod_id = 46599;
        let config = ConfigBuilder::default().profile(game).build().unwrap();

        let cache = Cache::new(&config).await.unwrap();
        let client = Client::new(&config).await;
        let logger = Logger::default();
        let updater = UpdateChecker::new(cache.clone(), client, config, logger);

        match updater.refresh_filelist(game, mod_id).await {
            Ok(_fl) => panic!("Refresh should have failed"),
            Err(e) => match e {
                ApiError::IsUnitTest => Ok(()),
                _ => {
                    panic!("Refresh should return ApiError::IsUnitTest");
                }
            },
        }
    }

    #[tokio::test]
    async fn up_to_date() -> Result<(), ApiError> {
        let game = "morrowind";
        let upload_time = 1310405800;
        let mod_id = 39350;
        let _fair_magicka_regen_file_id = 82041;

        let config = ConfigBuilder::default().profile(game).build().unwrap();
        let cache = Cache::new(&config).await?;
        let msgs = Logger::default();
        let client = Client::new(&config).await;
        let update = UpdateChecker::new(cache.clone(), client, config, msgs);

        let lock = cache.file_index.mod_file_map.read().await;
        let files = lock.get(&(game.to_string(), mod_id)).unwrap();
        let file_list = cache.file_lists.get((game, mod_id)).await.unwrap();
        let checked = update.check_mod(files, &file_list).await;

        match checked.first().unwrap().1 {
            UpdateStatus::UpToDate(t) => {
                if t == upload_time {
                    return Ok(());
                } else {
                    panic!("File had correct status but incorrect time {t}, expected {upload_time}.");
                }
            }
            _ => {
                panic!("File should be up to date");
            }
        }
    }

    #[tokio::test]
    async fn out_of_date() -> Result<(), ApiError> {
        let game = "morrowind";
        let mod_id = 46599;
        let _graphic_herbalism_file_id = 1000014314;
        let newest_file_update = 1558643755;

        let latest_local_time = 1558643754;
        let latest_remote_time = 1558643755;

        let config = ConfigBuilder::default().profile(game).build().unwrap();
        let cache = Cache::new(&config).await?;
        let msgs = Logger::default();
        let client = Client::new(&config).await;
        let update = UpdateChecker::new(cache.clone(), client, config, msgs);

        let lock = cache.file_index.mod_file_map.read().await;
        let files = lock.get(&(game.to_string(), mod_id)).unwrap();
        let file_list = cache.file_lists.get((game, mod_id)).await.unwrap();
        let checked = update.check_mod(files, &file_list).await;

        for f in checked {
            //println!("{}, {}", f.0.file_details.name, f.1.time());
            match f {
                (_, UpdateStatus::OutOfDate(t)) => {
                    assert_eq!(t, latest_local_time);
                    assert_eq!(newest_file_update, latest_remote_time);
                }
                (file, _) => {
                    panic!("UpdateStatus should be OutOfDate: {}", file.file_details.name);
                }
            }
        }
        Ok(())
    }
}
