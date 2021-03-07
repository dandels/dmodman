use super::Cacheable;
use crate::api::{ { Client, FileList, Queriable} , error::RequestError };
use super::error::*;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use super::cache::Cache;
use super::local_file::*;

pub struct UpdateChecker {
    pub updatable_mods: HashSet<u32>,
    pub file_lists: HashMap<u32, FileList>,
}

impl UpdateChecker {
    #[cfg(test)]
    pub fn new_with_file_lists(file_lists: HashMap<u32, FileList>) -> Self {
        Self {
            updatable_mods: HashSet::new(),
            file_lists
        }
    }

    pub fn new() -> Self {
        Self {
            updatable_mods: HashSet::new(),
            file_lists: HashMap::new(),
        }
    }

    pub async fn check_all(&mut self, client: &Client) -> Result<&HashSet<u32>, UpdateError> {
        self.check_files(client).await
    }

    pub async fn check_files(&mut self, client: &Client) -> Result<&HashSet<u32>, UpdateError> {
        for lf in client.cache.local_files.read().unwrap().clone().into_iter() {
            if self.check_file(client, &lf).await? {
                self.updatable_mods.insert(lf.mod_id);
            }
        }

        Ok(&self.updatable_mods)
    }

    pub async fn check_file(&self, client: &Client, local_file: &LocalFile) -> Result<bool, RequestError> {
        /* - Find out the mod for this file
         * - If the mod is already checked, return that result
         * - Otherwise loop through the file update history
         */

         if self.updatable_mods.contains(&local_file.mod_id) {
             return Ok(true)
         }

         let mut file_list;
         match self.file_lists.get(&local_file.mod_id) {
             Some(v) => file_list = v.to_owned(),
             None => {
                 println!("{:?}", self.file_lists);

                 // TODO handle files from other mods gracefully, eg. Skyrim SSE + Oldrim
                 file_list = FileList::request(client, vec![&local_file.game, &local_file.mod_id.to_string()]).await?;
                 file_list.save_to_cache(&local_file.game, &local_file.mod_id)?;
                 file_list.file_updates.sort_by_key(|a| a.uploaded_timestamp);
            }
         }

         Ok(self.file_has_update(&local_file, &file_list))
    }

    fn file_has_update(&self, local_file: &LocalFile, file_list: &FileList) -> bool {
        let mut has_update = false;
        let mut current_id = local_file.file_id;
        let mut latest_file: &str = &local_file.file_name;

         /* This could be an infinite loop if the data is corrupted and the file id's point to
          * eachother recursively. Using a for-loop fixes that, but doesn't do anything to fix the
          * error.
         */
        for _ in 0..file_list.file_updates.len() {
            match file_list
                .file_updates
                .iter()
                .find(|x| x.old_file_id == current_id)
            {
                Some(v) => {
                    /* If new_file_name matches a file on disk, then there are multiple downloads of
                     * the same mod, and we're currently looking at the old version.
                     * This doesn't yet mean that there are updates - the user can have an old version
                     * and the newest version.
                     */
                    current_id = v.new_file_id;
                    latest_file = &v.new_file_name;
                    has_update = true;
                }
                /* We've reached the end of the update history for one file. If the latest file doesn't
                 * exist, this file is assumed to have updates.
                 * Note that the API can't be trusted to remember every old file.
                 */
                None => {
                    let mut f: PathBuf = local_file.path().parent().unwrap().to_path_buf();
                    f.push(latest_file);
                    return !Path::new(&f).exists() && has_update;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::ErrorList;
    use crate::api::{ Client, FileList  };
    use crate::db::update::{ UpdateChecker, UpdateError };
    use crate::db::{ Cache, Cacheable };
    use crate::test;
    use std::collections::HashMap;

    #[tokio::test]
    async fn update() -> Result<(), UpdateError> {
        test::setup();
        let game: String = "morrowind".to_owned();

        let herba_id = 46599;
        let magicka_id = 39350;

        let mut file_lists: HashMap<u32, FileList> = HashMap::new();

        let herba_list = FileList::try_from_cache(&game, &herba_id).unwrap();
        let magicka_list = FileList::try_from_cache(&game, &magicka_id).unwrap();
        file_lists.insert(herba_id, herba_list);
        file_lists.insert(magicka_id, magicka_list);

        let cache = Cache::new(&game)?;
        let errors = ErrorList::default();
        let client: Client = Client::new(&cache, &errors)?;

        let mut updater = UpdateChecker::new_with_file_lists(file_lists);
        let upds = updater.check_all(&client).await?;

        println!("{:?}", upds);

        assert_eq!(true, upds.contains(&herba_id));
        assert_eq!(false, upds.contains(&magicka_id));
        Ok(())
    }
}
