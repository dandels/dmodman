use super::local_file::*;
use crate::api::{
    error::DownloadError,
    {Client, FileList, Queriable},
};
use crate::Messages;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::task;

#[derive(Clone)]
pub struct UpdateChecker {
    pub updatable: Arc<RwLock<HashMap<(String, u32), Vec<LocalFile>>>>,
    pub client: Client,
    msgs: Messages,
}

impl UpdateChecker {
    pub fn new(client: Client) -> Self {
        Self {
            updatable: Arc::new(RwLock::new(HashMap::new())),
            msgs: client.msgs.clone(),
            client,
        }
    }

    pub async fn check_all(&self) -> Result<(), DownloadError> {
        let mut mods_to_check: HashMap<(String, u32), Vec<LocalFile>> = HashMap::new();
        for lf in self.client.cache.local_files.try_read().unwrap().clone().into_iter() {
            match mods_to_check.get_mut(&(lf.game.clone(), lf.mod_id)) {
                Some(vec) => vec.push(lf.clone()),
                None => {
                    let mut vec = Vec::new();
                    vec.push(lf.clone());
                    mods_to_check.insert((lf.game, lf.mod_id), vec);
                }
            }
        }

        let mut handles: Vec<task::JoinHandle<Result<(), DownloadError>>> = Vec::new();
        for ((game, mod_id), files) in mods_to_check {
            let me = self.clone();
            let handle: task::JoinHandle<Result<(), DownloadError>> = task::spawn(async move {
                let upds = me.check_mod(&game, mod_id, files).await?;
                me.updatable.write().unwrap().insert((game, mod_id), upds);
                Ok(())
            });
            handles.push(handle);
        }
        for h in handles {
            match h.await {
                Ok(_) => {}
                Err(e) => self.msgs.push(e.to_string()),
            }
        }

        Ok(())
    }

    pub async fn check_mod(
        &self,
        game: &str,
        mod_id: u32,
        files: Vec<LocalFile>,
    ) -> Result<Vec<LocalFile>, DownloadError> {
        /* We might be able to tell that a file needs updates using the cached filelist, but it's not certain. First we
         * check the local version - if it doesn't have updates, we query the API.
         */
        let mut have_updates = Vec::new();
        let mut needs_refresh = false;
        match self.client.cache.file_lists.get(&game, mod_id) {
            Some(mut fl) => {
                fl.file_updates.sort_by_key(|a| a.uploaded_timestamp);
                for lf in files.clone() {
                    if self.file_has_update(&lf, &fl) {
                        have_updates.push(lf);
                        needs_refresh = false;
                    }
                }
            }
            None => needs_refresh = true,
        }

        if needs_refresh {
            let mut file_list = FileList::request(&self.client, vec![&game, &mod_id.to_string()]).await?;
            self.client.cache.save_file_list(&file_list, &game, &mod_id).await?;
            file_list.file_updates.sort_by_key(|a| a.uploaded_timestamp);

            for lf in files {
                if self.file_has_update(&lf, &file_list) {
                    have_updates.push(lf);
                }
            }
        }
        Ok(have_updates)
    }

    fn file_has_update(&self, local_file: &LocalFile, file_list: &FileList) -> bool {
        let mut has_update = false;
        let mut current_id = local_file.file_id;
        let mut latest_file: &str = &local_file.file_name;

        // For this to work the file updates need to be sorted by key. It's up to the caller to do so to preserve
        file_list.file_updates.iter().for_each(|x| {
            if x.old_file_id == current_id {
                current_id = x.new_file_id;
                latest_file = &x.new_file_name;
                has_update = true;
            }
        });
        let mut f: PathBuf = local_file.path().parent().unwrap().to_path_buf();
        f.push(latest_file);
        return !Path::new(&f).exists() && has_update;
    }
}

#[cfg(test)]
mod tests {
    use crate::api::Client;
    use crate::cache::update::{DownloadError, UpdateChecker};
    use crate::cache::Cache;
    use crate::Messages;

    #[tokio::test]
    async fn update() -> Result<(), DownloadError> {
        let game: String = "morrowind".to_owned();

        let herba_id = 46599;
        let magicka_id = 39350;

        let cache = Cache::new(&game).await?;
        let msgs = Messages::default();
        let client: Client = Client::new(&cache, &msgs)?;

        let updater = UpdateChecker::new(client);
        updater.check_all().await?;

        let upds = updater.updatable.read().unwrap();
        assert_eq!(false, upds.get(&(game.clone(), magicka_id)).unwrap().first().is_some());

        assert_eq!(
            upds.get(&(game.clone(), herba_id)).unwrap().get(0).unwrap().file_name,
            "Graphic Herbalism MWSE - OpenMW-46599-1-03-1556986083.7z"
        );

        assert_eq!(
            upds.get(&(game, herba_id)).unwrap().get(1).unwrap().file_name,
            "GH TR - PT Meshes-46599-1-01-1556986716.7z"
        );

        Ok(())
    }
}
