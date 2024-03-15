use super::UpdateStatus;
use super::{Client, FileList, Queriable};
use crate::api::{Query, Updated};
use crate::cache::{Cache, ModFileMetadata};
use crate::Config;
use crate::Logger;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task;

#[derive(Clone)]
pub struct UpdateChecker {
    cache: Cache,
    client: Client,
    config: Arc<Config>,
    logger: Logger,
    query: Query,
}

impl UpdateChecker {
    pub fn new(cache: Cache, client: Client, config: Arc<Config>, logger: Logger, query: Query) -> Self {
        Self {
            cache,
            client,
            config,
            logger,
            query,
        }
    }

    pub async fn ignore_file(&self, file_id: u64) {
        if let Some(mfd) = self.cache.metadata_index.get_by_file_id(&file_id).await {
            if let Some(latest_remote_file) =
                self.cache.file_lists.get(mfd.game.clone(), mfd.mod_id).await.unwrap().file_updates.last()
            {
                match mfd.update_status.to_enum() {
                    UpdateStatus::OutOfDate(_) => {
                        mfd.propagate_update_status(
                            &self.config,
                            &self.logger,
                            &UpdateStatus::IgnoredUntil(latest_remote_file.uploaded_timestamp),
                        )
                        .await;
                    }
                    UpdateStatus::HasNewFile(_) => {
                        mfd.propagate_update_status(
                            &self.config,
                            &self.logger,
                            &UpdateStatus::UpToDate(latest_remote_file.uploaded_timestamp),
                        )
                        .await;
                    }
                    _ => {}
                }
                self.cache.archives.has_changed.store(true, Ordering::Relaxed);
                self.cache.installed.has_changed.store(true, Ordering::Relaxed);
            }
        }
    }

    pub async fn update_all(&self) {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            // If less than a month has passed since previous update we can use the API endpoint for mod updates
            Ok(time) => {
                // TODO the updated timestamp is per profile and doesn't take into account user moved files
                let t_diff = time.as_secs() - self.cache.last_update_check.load(Ordering::Relaxed);
                // this is how many seconds are in 28 days
                if t_diff < 2419200 {
                    let me = self.clone();
                    task::spawn(async move {
                        let mods_by_game = me.cache.metadata_index.by_game_and_mod_sorted.read().await;
                        if let Err(e) = me.cache.save_last_updated(time.as_secs()).await {
                            me.logger.log(format!("Failed to save last updated status: {}", e));
                        }

                        /* The updated mod lists are provided per game and sorted by mod id
                         * FileIndex.game_to_mods_map  */

                        for (game, mod_map) in mods_by_game.iter() {
                            match Updated::request(&me.client, &[&game]).await {
                                Ok(updated_mods) => {
                                    // Uncomment to save Updated lists
                                    //if let Err(e) =
                                    //    updated_mods.save(me.config.path_for(DataType::Updated(&game))).await
                                    //{
                                    //    me.logger.log(format!("Unable to save update list for {game}: {}", e));
                                    //}

                                    // Local and updated mods are sorted so we can iterate in parallel
                                    for (mod_id, files) in mod_map {
                                        for upd in &updated_mods.updates {
                                            if upd.mod_id == *mod_id {
                                                me.update_mod(game.clone(), *mod_id, files.clone()).await;
                                            }
                                            if upd.mod_id > *mod_id {
                                                break;
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    me.logger.log(format!("Unable to fetch update lists for {game}: {}", e));
                                    return;
                                }
                            }
                        }
                        if let Err(e) = me.cache.save_last_updated(time.as_secs()).await {
                            me.logger.log(format!("Failed to save last_updated: {e}"));
                        }
                    });
                } else {
                    self.logger.log("Over a month since last update check, checking each mod.");
                    for (game, mods) in self.cache.metadata_index.by_game_and_mod_sorted.read().await.iter() {
                        for (mod_id, files) in mods {
                            self.update_mod(game.clone(), *mod_id, files.clone()).await;
                        }
                    }
                }
            }
            // This is a ridiculous error case to handle, but avoids an unwrap()
            Err(e) => {
                self.logger.log(format!("WARNING: Refusing to update, system time is before Unix epoch: {}", e));
            }
        };
        self.logger.log("Finished checking updates.");
    }

    pub async fn update_mod(&self, game: String, mod_id: u32, files_in_mod: Vec<Arc<ModFileMetadata>>) {
        let me = self.clone();
        task::spawn(async move {
            let mut needs_refresh = false;
            let mut checked: Vec<(Arc<ModFileMetadata>, UpdateStatus)> = vec![];
            /* First try to check updates with cached values.
             * If the UpdateStatus is already OutOfDate or HasNewFile, there's no reason to query the API.
             * Only query the API if a file is still reported as UpToDate.
             */
            if let Some(fl) = me.cache.file_lists.get(game.clone(), mod_id).await {
                checked = me.check_mod(&files_in_mod, &fl).await;
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
                match me.query.file_list(&game, mod_id).await {
                    Ok(fl) => {
                        checked = me.check_mod(&files_in_mod, &fl).await;
                    }
                    Err(e) => {
                        me.logger.log(format!("Error when refreshing filelist for {mod_id}: {}", e));
                    }
                }
            }
            for (mfd, new_status) in checked {
                if mfd.update_status.to_enum() != new_status {
                    mfd.propagate_update_status(&me.config, &me.logger, &new_status).await;
                }
            }
            me.cache.archives.has_changed.store(true, Ordering::Relaxed);
            me.cache.installed.has_changed.store(true, Ordering::Relaxed);
        });
    }

    /* This is complicated and maybe buggy.
     *
     * There are several ways in which a mod can have updates.
     * 1) The FileList response for a mod contains a file_updates array, with which we can iterate over the update chain
     *    for a specific file id. The updates need to be sorted by timestamp or the time complexity of this is O(n²) per
     *    file.
     *    However, The NexusMods community manager, who has been very helpful, couldn't guarantee that the API keeps them
     *    sorted.
     *    To reduce the amount of time spent iterating over file lists, the file updates are sorted by timestamp when
     *    added to the cache (they should already be sorted, but verifying it takes O(n) time).
     *
     *    The list of the user's downloaded files is also sorted by timestamp (technically by file id since the time
     *    stamp could be missing, but the result should be the same).
     *
     *    We then iterate backwards over both lists at once. This allows us to skip iterating over any update that is
     *    older than our files. The next file to check can reuse the index of the previous file, reducing iteration time.
     *
     * 2) The category of the file might have changed to OLD_VERSION or ARCHIVED without the update list containing new
     *    versions of that file.
     *
     * 3) Even when neither of these are true, there might be some other new file in the mod. This could be an optional
     *    file, a patch for the main mod or between the mod and some other mod, a new version that doesn't fit in the
     *    previous two categories, etc. Since figuring out these cases is infeasible, we set timestamps on each file's
     *    UpdateStatus (setting it on the latest one isn't enough, as the user could delete it).
     *    If none of the other update conditions are true, we set the file's update status to UpToDate or HasNewFile */
    async fn check_mod(
        &self,
        to_check: &Vec<Arc<ModFileMetadata>>,
        file_list: &FileList,
    ) -> Vec<(Arc<ModFileMetadata>, UpdateStatus)> {
        if to_check.is_empty() {
            self.logger.log("Tried to check updates for nonexistent files. This shouldn't happen.");
            return vec![];
        }

        let updates = &file_list.file_updates;
        // careful to not go out of bounds using this
        let mut newer_updates_start_index = updates.len().checked_sub(1);
        let mut checked: Vec<(Arc<ModFileMetadata>, UpdateStatus)> = vec![];
        let latest_local_time = 't: {
            for mfd in to_check.iter().rev() {
                if let Some(t) = mfd.uploaded_timestamp().await {
                    break 't Some(t);
                }
            }
            None
        };
        // The last file in the file list should be the newest.
        let latest_remote_time = file_list.files.last().unwrap().uploaded_timestamp;

        for mfd in to_check.iter().rev() {
            let update_status = mfd.update_status.to_enum();
            //match update_status {
            //    // No need to check files that are already known to have updates
            //    UpdateStatus::OutOfDate(_) | UpdateStatus::HasNewFile(_) => {
            //        checked.push((mfd.clone(), update_status));
            //        continue;
            //    }
            //    _ => {}
            //}

            let mut has_update = false;

            // enums used by the API
            const OLD_VERSION: u32 = 4;
            const ARCHIVED: u32 = 7;
            if let Some(file_details) = mfd.file_details().await {
                if file_details.category_id == OLD_VERSION || file_details.category_id == ARCHIVED {
                    has_update = true;
                } else {
                    /* Note: Both collections are sorted by their upload timestamp.
                     * Get the range of files in the update lists that are newer than the file we're checking.
                     * We're iterating both the files to check and the update lists in reverse, and the next file to
                     * check can reuse the index of the previous newer_updates_index, because that file is older than
                     * this one. */
                    if let Some(index) = newer_updates_start_index {
                        while index > 0 {
                            let upd = &updates[index];
                            /* The timestamp in the file updates might be slightly later than the one in the FileList, so we
                             * also need to compare file_id's. */
                            if file_details.uploaded_timestamp < upd.uploaded_timestamp
                                && mfd.file_id != upd.new_file_id
                            {
                                newer_updates_start_index = Some(index - 1);
                            } else {
                                break;
                            }
                        }
                    }
                }
            }

            // Is this file marked as an old file in the updates chain?
            if let Some(index) = newer_updates_start_index {
                if !has_update {
                    for upd in &updates[index..] {
                        if mfd.file_id == upd.old_file_id {
                            has_update = true;
                            break;
                        }
                    }
                }
            }
            if has_update {
                match update_status {
                    // Set file out of date unless this update is ignored
                    UpdateStatus::IgnoredUntil(t) => {
                        if t < latest_remote_time {
                            checked.push((mfd.clone(), UpdateStatus::OutOfDate(latest_remote_time)));
                        // this is still ignored and we don't touch it
                        } else {
                            checked.push((mfd.clone(), UpdateStatus::IgnoredUntil(t)));
                        }
                    }
                    _ => {
                        checked.push((mfd.clone(), UpdateStatus::OutOfDate(latest_remote_time)));
                    }
                }
            // No direct update in update chain, but there might be new files
            } else if let Some(latest_local_time) = latest_local_time {
                if latest_local_time < latest_remote_time {
                    match update_status {
                        UpdateStatus::IgnoredUntil(t) => {
                            // another remote file has appeared since updates were ignored
                            if t < latest_remote_time {
                                checked.push((mfd.clone(), UpdateStatus::HasNewFile(latest_local_time)));
                            // this is still ignored and we don't touch it
                            } else {
                                checked.push((mfd.clone(), UpdateStatus::IgnoredUntil(t)));
                            }
                        }
                        _ => {
                            checked.push((mfd.clone(), UpdateStatus::HasNewFile(latest_local_time)));
                        }
                    }
                } else {
                    checked.push((mfd.clone(), UpdateStatus::UpToDate(latest_local_time)));
                }
            } else {
                /* If we get here we were unable to get the FileDetails of any of the users' downloaded files,
                 * but those files were still associated to a mod because they had a ModFileMetadata created for them.
                 * Refreshing the file list should have added the FileDetails to any mod missing it.
                 * If it still wasn't added then the file is probably so old that the API no longer lists it.
                 */
                if let Some(name) = mfd.name().await {
                    self.logger.log(format!("{}", name));
                }
                checked.push((mfd.clone(), UpdateStatus::OutOfDate(latest_remote_time)));
            }
        }
        checked
    }
}

#[cfg(test)]
mod tests {
    use super::UpdateStatus;
    use crate::api::{ApiError, Client, Query, UpdateChecker};
    use crate::cache::Cache;
    use crate::config::tests::setup_env;
    use crate::ConfigBuilder;
    use crate::Logger;
    use std::sync::Arc;

    async fn init_structs() -> (Cache, UpdateChecker) {
        let profile = "testprofile";
        let config = Arc::new(ConfigBuilder::default().profile(profile).build().unwrap());
        let logger = Logger::default();
        let cache = Cache::new(config.clone(), logger.clone()).await.unwrap();
        let client = Client::new(&config).await;
        let query = Query::new(cache.clone(), client.clone(), config.clone(), logger.clone());
        (cache.clone(), UpdateChecker::new(cache, client, config, logger, query))
    }

    // TODO this needs a lot more unit tests but they are rather tedious to create

    #[tokio::test]
    async fn up_to_date() -> Result<(), ApiError> {
        setup_env();
        let game = "morrowind";
        let upload_time = 1310405800;
        let mod_id = 39350;
        let _fair_magicka_regen_file_id = 82041;

        let (cache, update) = init_structs().await;

        let lock = cache.metadata_index.by_game_and_mod_sorted.read().await;
        let mod_map = lock.get(game).unwrap();
        let files = mod_map.get(&mod_id).unwrap();
        let file_list = cache.file_lists.get(game, mod_id).await.unwrap();
        let checked = update.check_mod(files, &file_list).await;

        match checked.first().unwrap().1 {
            UpdateStatus::UpToDate(t) => {
                if t == upload_time {
                    return Ok(());
                }
                panic!("File had correct status but incorrect time {t}, expected {upload_time}.");
            }
            _ => {
                panic!("File should be up to date");
            }
        }
    }

    #[tokio::test]
    async fn out_of_date() -> Result<(), ApiError> {
        setup_env();
        let game = "morrowind";
        let mod_id = 46599;
        let _graphic_herbalism_file_id = 1000014314;
        let newest_file_update = 1558643755;

        let latest_local_time = 1558643754;
        let latest_remote_time = 1558643755;

        let (cache, update) = init_structs().await;

        let lock = cache.metadata_index.by_game_and_mod_sorted.read().await;
        let mod_map = lock.get(game).unwrap();
        let files = mod_map.get(&mod_id).unwrap();
        let file_list = cache.file_lists.get(game, mod_id).await.unwrap();
        let checked = update.check_mod(files, &file_list).await;

        for (mfd, status) in checked {
            //println!("{}, {}", f.0.file_details.name, f.1.time());
            match status {
                UpdateStatus::OutOfDate(t) => {
                    assert_eq!(t, latest_local_time);
                    assert_eq!(newest_file_update, latest_remote_time);
                }
                _ => {
                    panic!("UpdateStatus should be OutOfDate for file with id: {}", mfd.file_id);
                }
            }
        }
        Ok(())
    }
}
