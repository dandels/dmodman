use super::error::DownloadError;
use super::{Client, FileList, Queriable};
use crate::cache::{Cache, Cacheable, LocalFile, UpdateStatus};
use crate::config::PathType;
use crate::Config;
use crate::Messages;

use std::collections::HashMap;

#[derive(Clone)]
pub struct UpdateChecker {
    client: Client,
    cache: Cache,
    config: Config,
    msgs: Messages,
}

impl UpdateChecker {
    pub fn new(cache: Cache, client: Client, config: Config, msgs: Messages) -> Self {
        Self {
            msgs,
            cache,
            client,
            config,
        }
    }

    pub async fn update_all(&self) -> Result<(), DownloadError> {
        // We only need to make one API request per mod, since the response contains info about all files in that mod.
        let mut games_to_check: HashMap<(String, u32), FileList> = HashMap::new();
        let mut localfiles = self.cache.file_index.map.write().await;

        for (lf, _fd) in localfiles.values_mut() {
            // TODO error handling
            match games_to_check.get_mut(&(lf.game.to_string(), lf.mod_id)) {
                Some(file_list) => {
                    lf.update_status = check_file(lf, file_list).await;
                    lf.save(self.config.path_for(PathType::LocalFile(lf))).await.unwrap();
                }
                None => {
                    let file_list = self.refresh_filelist(&lf.game, lf.mod_id).await.unwrap();
                    lf.update_status = check_file(lf, &file_list).await;
                    games_to_check.insert((lf.game.to_string(), lf.mod_id), file_list);
                    lf.save(self.config.path_for(PathType::LocalFile(lf))).await.unwrap();
                }
            }
        }
        Ok(())
    }

    pub async fn update_file(&self, lf: &mut LocalFile) {
        // TODO don't unwrap
        let file_list = self.refresh_filelist(&lf.game, lf.mod_id).await.unwrap();
        check_file(lf, &file_list).await;
    }

    async fn refresh_filelist(&self, game: &str, mod_id: u32) -> Result<FileList, DownloadError> {
        let file_list = FileList::request(&self.client, self.msgs.clone(), vec![game, &mod_id.to_string()]).await?;
        /* The update algorithm in check_file() requires the file list to be sorted.
         * The NexusMods community manager (who has been Very Helpful!) couldn't guarantee that the API always
         * keeps them sorted */
        self.cache.save_file_list(&file_list, game, mod_id).await?;
        Ok(file_list)
    }
}

/* There might be several versions of a file present. If we're looking at the oldest one, it's not enough to
 * check if a newer version exists. Instead we go through the file's versions, and return true if the newest one
 * doesn't exist.
 * TODO this needs a lot of unit tests.
 */
async fn check_file(local_file: &LocalFile, file_list: &FileList) -> UpdateStatus {
    let status = local_file.update_status.to_owned();

    if file_list.file_updates.is_empty() {
        return UpdateStatus::UpToDate(status.time());
    }
    let latest_file = file_list.file_updates.peek().unwrap();
    let latest_timestamp: u64 = latest_file.uploaded_timestamp;

    if local_file.file_name == latest_file.new_file_name {
        return UpdateStatus::UpToDate(latest_timestamp);
    }

    // This is unexpected, let's not do anything
    if latest_timestamp <= status.time() {
        return status;
    }

    let mut has_update = false;
    let mut current_id = local_file.file_id;
    let mut current_file: &str = &local_file.file_name;

    file_list.file_updates.iter().for_each(|x| {
        if x.old_file_id == current_id {
            current_id = x.new_file_id;
            current_file = &x.new_file_name;
            has_update = true;
        }
    });

    if has_update {
        UpdateStatus::OutOfDate(latest_timestamp)
    } else if status.time() < latest_timestamp {
        UpdateStatus::HasNewFile(latest_timestamp)
    } else {
        UpdateStatus::UpToDate(latest_timestamp)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::update;
    use crate::api::{Client, DownloadError, UpdateChecker};
    use crate::cache::Cache;
    use crate::cache::UpdateStatus;
    use crate::ConfigBuilder;
    use crate::Messages;

    #[tokio::test]
    #[should_panic]
    async fn block_test_request() {
        let game = "morrowind";
        let config = ConfigBuilder::default().game(game).build().unwrap();

        let cache = Cache::new(&config).await.unwrap();

        let msgs = Messages::default();
        let client = Client::new(&cache, &config, &msgs).await.unwrap();

        let msgs = Messages::default();
        let updater = UpdateChecker::new(cache.clone(), client, config, msgs);

        // TODO assert that res is RequestError::IsUnitTest
        let _res = updater.update_all().await;
    }

    #[tokio::test]
    async fn out_of_date() -> Result<(), DownloadError> {
        let game = "morrowind";
        let upload_time = 1310405800;
        let fair_magicka_regen_file_id = 82041;

        let config = ConfigBuilder::default().game(game).build().unwrap();
        let cache = Cache::new(&config).await?;

        let index = cache.file_index.map.read().await;
        let (fmr_lf, _fmr_fd) = index.get(&fair_magicka_regen_file_id).unwrap();
        let fmr_fl = cache.file_lists.get((&fmr_lf.game, fmr_lf.mod_id)).await;
        let status = update::check_file(fmr_lf, &fmr_fl.unwrap()).await;

        match status {
            UpdateStatus::UpToDate(t) => {
                assert_eq!(t, upload_time)
            }
            _ => {
                panic!("Mod should be up to date: {}", fmr_lf.file_name);
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn up_to_date() -> Result<(), DownloadError> {
        let game = "morrowind";
        let graphic_herbalism_file_id = 1000014314;
        let newest_file_update = 1558643755;

        let config = ConfigBuilder::default().game(game).build().unwrap();
        let cache = Cache::new(&config).await?;

        let index = cache.file_index.map.read().await;

        let (gh_lf, _gh_fd) = index.get(&graphic_herbalism_file_id).unwrap();
        let gh_fl = cache.file_lists.get((&gh_lf.game, gh_lf.mod_id)).await;

        let status = update::check_file(gh_lf, &gh_fl.unwrap()).await;
        match status {
            UpdateStatus::OutOfDate(t) => {
                assert_eq!(t, newest_file_update);
            }
            _ => {
                panic!("Mod should be out of date: {}", gh_lf.file_name);
            }
        };
        Ok(())
    }
}
