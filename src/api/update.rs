use super::error::RequestError;
use super::{Client, FileList, FileUpdate, Queriable};
use crate::cache::{Cache, Cacheable, FileData, UpdateStatus};
use crate::config::PathType;
use crate::Config;
use crate::Messages;

use std::collections::BinaryHeap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[derive(Clone)]
pub struct UpdateChecker {
    cache: Cache,
    client: Client,
    config: Config,
    msgs: Messages,
}

impl UpdateChecker {
    pub fn new(cache: Cache, client: Client, config: Config, msgs: Messages) -> Self {
        Self {
            cache,
            client,
            config,
            msgs,
        }
    }

    pub async fn update_all(&self) {
        // TODO reconsider at which point(s) of the type hierarchy the rwlock needs to be
        for ((game, mod_id), files) in self.cache.files.mod_files.read().await.iter() {
            self.msgs.push(format!("{game}, {mod_id}")).await;
            let mut needs_refresh = false;
            let mut checked: Vec<(Arc<FileData>, UpdateStatus)> = vec![];
            if let Some(fl) = self.cache.file_lists.get((game, *mod_id)).await {
                checked = self.check_mod(files, &fl).await;
                for (_fdata, status) in &checked {
                    if let UpdateStatus::UpToDate(_) = status {
                        needs_refresh = true;
                    }
                }
            } else {
                self.msgs.push(format!("Strange, no file list in cache for {mod_id}. Fetching.")).await;
                needs_refresh = true;
            }
            if needs_refresh {
                /* We only need to make one API request per mod, since the response contains info about all files in
                 * that mod. */
                match self.refresh_filelist(game, *mod_id).await {
                    Ok(fl) => {
                        checked = self.check_mod(files, &fl).await;
                    }
                    Err(e) => {
                        self.msgs.push(format!("Error when refresh filelist for {mod_id}: {}", e)).await;
                    }
                }
            }
            for (file, new_status) in checked {
                self.msgs.push(format!("setting {} status to {:?}", file.file_details.name, new_status)).await;
                let mut lf = file.local_file.write().await;
                if lf.update_status != new_status {
                    lf.update_status = new_status;
                    lf.save(self.config.path_for(PathType::LocalFile(&lf))).await.unwrap();
                }
            }
            self.cache.files.has_changed.store(true, Ordering::Relaxed);
        }
    }

    async fn refresh_filelist(&self, game: &str, mod_id: u32) -> Result<FileList, RequestError> {
        let file_list = FileList::request(&self.client, self.msgs.clone(), vec![game, &mod_id.to_string()]).await?;
        /* The update algorithm in check_file() requires the file list to be sorted.
         * The NexusMods community manager (who has been Very Helpful!) couldn't guarantee that the API always
         * keeps them sorted */
        self.cache.save_file_list(&file_list, game, mod_id).await?;
        Ok(file_list)
    }

    async fn check_mod(
        &self,
        to_check: &BinaryHeap<Arc<FileData>>,
        file_list: &FileList,
    ) -> Vec<(Arc<FileData>, UpdateStatus)> {
        if to_check.peek().is_none() {
            self.msgs.push("Tried to check updates for nonexistent files. This shouldn't happen.").await;
            return vec![];
        }

        {
            self.msgs
                .push(format!(
                    "Checking: {}",
                    to_check.peek().unwrap().local_file.read().await.game
                ))
                .await;
        }

        let mut files = to_check.clone();
        let mut updates = file_list.file_updates.clone();
        let mut checked: Vec<(Arc<FileData>, UpdateStatus)> = vec![];
        let latest_local_time = to_check.peek().unwrap().file_details.uploaded_timestamp;
        let latest_remote_time = file_list.files.last().unwrap().uploaded_timestamp;

        let mut newer_files: Vec<FileUpdate> = vec![];
        while let Some(file) = files.pop() {
            self.msgs.push(format!("popping {:?}", file.file_details.file_name)).await;
            let mut has_update = false;

            const OLD_VERSION: u32 = 4;
            const ARCHIVED: u32 = 7;
            if file.file_details.category_id == OLD_VERSION || file.file_details.category_id == ARCHIVED {
                has_update = true;
            } else {
                /* For each file we're checking, we're only concerned about files that are newer than it.
                 * Files that we iterate on after this one can reuse this same information, since both heaps are sorted by
                 * timestamp. */
                while let Some(upd) = updates.peek() {
                    if file.file_details.uploaded_timestamp < upd.uploaded_timestamp {
                        self.msgs.push(format!("    newer than us: {:?}", &upd.new_file_name)).await;
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
                    self.msgs.push(format!("Found {} is old", &file.file_details.name)).await;
                    break;
                }
            }
            let local_file = file.local_file.read().await;
            // Set file out of date unless this update is ignored
            if has_update {
                match local_file.update_status {
                    UpdateStatus::IgnoredUntil(t) => {
                        if t < latest_remote_time {
                            checked.push((file.clone(), UpdateStatus::OutOfDate(latest_remote_time)));
                        } else {
                            // else this is ignored and we return it as it was
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
                        if t < latest_remote_time {
                            checked.push((file.clone(), UpdateStatus::HasNewFile(latest_local_time)));
                        } else {
                            // else this is ignored and we return it as it was
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
    use crate::api::{Client, RequestError, UpdateChecker};
    use crate::cache::Cache;
    use crate::cache::UpdateStatus;
    use crate::ConfigBuilder;
    use crate::Messages;

    #[tokio::test]
    async fn block_test_request() -> Result<(), RequestError> {
        let game = "morrowind";
        let mod_id = 46599;
        let config = ConfigBuilder::default().game(game).build().unwrap();

        let cache = Cache::new(&config).await.unwrap();
        let msgs = Messages::default();
        let client = Client::new(&cache, &config, &msgs).await;
        let msgs = Messages::default();
        let updater = UpdateChecker::new(cache.clone(), client, config, msgs);

        match updater.refresh_filelist(game, mod_id).await {
            Ok(_fl) => panic!("Refresh should have failed"),
            Err(e) => match e {
                RequestError::IsUnitTest => Ok(()),
                _ => {
                    panic!("Refresh should return RequestError::IsUnitTest");
                }
            },
        }
    }

    #[tokio::test]
    async fn up_to_date() -> Result<(), RequestError> {
        let game = "morrowind";
        let upload_time = 1310405800;
        let mod_id = 39350;
        let _fair_magicka_regen_file_id = 82041;

        let config = ConfigBuilder::default().game(game).build().unwrap();
        let cache = Cache::new(&config).await?;
        let msgs = Messages::default();
        let client = Client::new(&cache, &config, &msgs).await;
        let update = UpdateChecker::new(cache.clone(), client, config, msgs);

        let lock = cache.files.mod_files.read().await;
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
    async fn out_of_date() -> Result<(), RequestError> {
        panic!("todo");
        //let game = "morrowind";
        //let mod_id = 46599;
        //let graphic_herbalism_file_id = 1000014314;
        //let newest_file_update = 1558643755;

        //let config = ConfigBuilder::default().game(game).build().unwrap();
        //let cache = Cache::new(&config).await?;
        //let msgs = Messages::default();
        //let client = Client::new(&cache, &config, &msgs).await;
        //let update = UpdateChecker::new(cache.clone(), client, config, msgs);

        ////let index = cache.files.file_index.read().await;

        ////let (gh_lf, _gh_fd) = index.get(&graphic_herbalism_file_id).unwrap();
        ////let gh_fl = cache.file_lists.get((&gh_lf.game, gh_lf.mod_id)).await;

        ////let status = update::check_file(gh_lf, &gh_fl.unwrap()).await;

        //let lock = cache.files.mod_files.read().await;
        //let files = lock.get(&(game.to_string(), mod_id)).unwrap();
        //let file_list = cache.file_lists.get((game, mod_id)).await.unwrap();
        //let checked = update.check_mod(files, &file_list).await;
        //match status {
        //    UpdateStatus::OutOfDate(t) => {
        //        assert_eq!(t, newest_file_update);
        //    }
        //    _ => {
        //        panic!("Mod should be out of date: {}", gh_lf.file_name);
        //    }
        //};
        //Ok(())
    }
}
