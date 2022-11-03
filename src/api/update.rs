use super::error::DownloadError;
use super::{Client, FileList, Queriable};
use crate::cache::{Cache, LocalFile, UpdateStatus};
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

    pub async fn update_all(&self) {
        // We only need to make one API request per mod, since the response contains info about all files in that mod.
        let mut games_to_check: HashMap<(String, u32), FileList> = HashMap::new();
        let mut localfiles = self.cache.file_index.map.write().await;

        for (lf, _fd) in localfiles.values_mut() {
            match games_to_check.get_mut(&(lf.game.to_string(), lf.mod_id)) {
                Some(file_list) => {
                    self.check_file(lf, file_list).await;
                }
                None => {
                    // TODO don't unwrap
                    let file_list = self.refresh_filelist(&lf.game, lf.mod_id).await.unwrap();
                    self.check_file(lf, &file_list).await;
                    games_to_check.insert((lf.game.to_string(), lf.mod_id), file_list);
                }
            }
        }
    }

    pub async fn update_file(&self, lf: &mut LocalFile) {
        // TODO don't unwrap
        let file_list = self.refresh_filelist(&lf.game, lf.mod_id).await.unwrap();
        self.check_file(lf, &file_list).await;
    }

    async fn refresh_filelist(&self, game: &str, mod_id: u32) -> Result<FileList, DownloadError> {
        let mut file_list = FileList::request(&self.client, self.msgs.clone(), vec![game, &mod_id.to_string()]).await?;
        /* The update algorithm in file_has_update() requires the file list to be sorted.
         * The NexusMods community manager (who has been Very Helpful!) couldn't guarantee that the API always
         * keeps them sorted */
        file_list.file_updates.sort_by_key(|a| a.uploaded_timestamp);
        self.cache.save_file_list(&file_list, game, mod_id).await?;
        Ok(file_list)
    }

    /* There might be several versions of a file present. If we're looking at the oldest one, it's not enough to
     * check if a newer version exists. Instead we go through the file's versions, and return true if the newest one
     * doesn't exist.
     * TODO return status for easier unit testing, change LocalFile outside this function
     * TODO comment what's going on.
     * TODO this needs a lot of unit tests.
     */
    async fn check_file(&self, local_file: &mut LocalFile, file_list: &FileList) {
        if file_list.file_updates.is_empty() {
            return;
        }
        let latest_timestamp: u64 = file_list.file_updates.last().unwrap().uploaded_timestamp;

        if let Some(UpdateStatus::IgnoredUntil(ignoretime)) = local_file.update_status {
            if latest_timestamp < ignoretime {
                return;
            }
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

        let mut f = self.config.path_for(PathType::LocalFile(local_file)).parent().unwrap().to_path_buf();
        f.push(current_file);
        // TODO does it matter if the file exists? We don't need to check from disk
        match f.try_exists() {
            Ok(false) => {
                local_file.update_status = Some(UpdateStatus::OutOfDate);
            }
            Ok(true) => {
                if has_update {
                    if let Some(UpdateStatus::IgnoredUntil(t)) = local_file.update_status {
                        if t < latest_timestamp {
                            local_file.update_status = Some(UpdateStatus::OutOfDate);
                        } else {
                            // do nothing
                        }
                    } else {
                        local_file.update_status = Some(UpdateStatus::OutOfDate);
                    }
                } else if let Some(UpdateStatus::UpToDate(previous_timestamp)) = local_file.update_status {
                    if previous_timestamp < latest_timestamp {
                        local_file.update_status = Some(UpdateStatus::HasNewFile(latest_timestamp));
                    } else {
                        local_file.update_status = Some(UpdateStatus::UpToDate(latest_timestamp));
                    }
                } else {
                    local_file.update_status = Some(UpdateStatus::UpToDate(latest_timestamp));
                }
            }
            Err(e) => {
                self.msgs
                    .push(format!(
                        "IO error when checking update for {:?}: {e:?}",
                        local_file.file_name
                    ))
                    .await;
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{Client, DownloadError, UpdateChecker};
    use crate::cache::Cache;
    use crate::cache::UpdateStatus;
    use crate::ConfigBuilder;
    use crate::Messages;

    // TODO this currently fails since it tries to send API request
    #[tokio::test]
    async fn update() -> Result<(), DownloadError> {
        let game = "morrowind";
        let config = ConfigBuilder::default().game(game).build().unwrap();

        let fair_magicka_regen_file_id = 82041;
        let graphic_herbalism_file_id = 1000014314;

        let cache = Cache::new(&config).await?;
        let msgs = Messages::default();
        let client: Client = Client::new(&cache, &config, &msgs).await?;

        let msgs = Messages::default();
        let updater = UpdateChecker::new(cache.clone(), client, config, msgs);

        updater.update_all().await;
        let index = cache.file_index.map.read().await;
        let (fmr_lf, _fd) = index.get(&fair_magicka_regen_file_id).unwrap();

        let (gh_lf, _fd) = index.get(&graphic_herbalism_file_id).unwrap();

        assert!(matches!(fmr_lf.clone().update_status.unwrap(), UpdateStatus::OutOfDate));
        assert!(matches!(gh_lf.clone().update_status.unwrap(), UpdateStatus::OutOfDate));

        Ok(())
    }
}
