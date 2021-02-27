use crate::api::{ {FileList, Requestable, Cacheable}, error::RequestError };
use crate::config;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use super::LocalFile;

pub struct UpdateChecker {
    pub game: String,
    pub updatable_mods: HashSet<u32>,
    pub file_lists: HashMap<u32, FileList>,
}

impl UpdateChecker {
    pub fn new(game: &str) -> Self {
        Self {
        game: game.to_string(),
        updatable_mods: HashSet::new(),
        file_lists: HashMap::new(),
        }
    }

    pub async fn check_all(&self) -> Result<&HashSet<u32>, RequestError> {
        // Selects files in download directory that end with .json
        let jsonfiles = fs::read_dir(config::download_dir(&self.game))?.flatten().map(|x|
            if x.path().is_file() && x.path().extension().and_then(OsStr::to_str) == Some("json") { 
                Some(x.path())
            } else {
                None 
            }
        ).flatten();

        for file in jsonfiles {
            self.check_file(&file).await?;
        }

        Ok(&self.updatable_mods)
    }

    // Is this needed?
    pub fn check_mod(&self, mod_id: &u32) {
        unimplemented!();
    }

    pub async fn check_file(&self, path: &Path) -> Result<bool, RequestError> {
        /* - Find out the mod for this file
         * - If the mod is already checked, return that result
         * - Otherwise loop through the file update history
         */

         let local_file: LocalFile = serde_json::from_str(&std::fs::read_to_string(&path)?).unwrap_or_else(
             |_| panic!("Unable to deserialize metadata for {:?}", path)
             );

         if self.updatable_mods.contains(&local_file.mod_id) {
             return Ok(true)
         }

         let mut file_list;
         match self.file_lists.get(&local_file.mod_id) {
             Some(v) => file_list = v.to_owned(),
             None => {
                 // TODO handle files from other mods gracefully, eg. Skyrim SSE + Oldrim
                 file_list = FileList::request(vec![&local_file.game, &local_file.mod_id.to_string()]).await?;
                 file_list.save_to_cache(&local_file.game, &local_file.mod_id)?;
                 file_list.file_updates.sort_by_key(|a| a.uploaded_timestamp);
            }
         }

         Ok(self.file_has_update(&local_file, &path, &file_list))
    }

    fn file_has_update(&self, local_file: &LocalFile, path: &Path, file_list: &FileList) -> bool {
        let mut has_update = false;
        let mut current_id = local_file.file_id;
        let mut latest_file: String = path.to_str().unwrap().to_owned();

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
                    latest_file = v.new_file_name.clone();
                    has_update = true;
                }
                /* We've reached the end of the update history for one file. If the latest file doesn't
                 * exist, this file is assumed to have updates.
                 * Note that the API can't be trusted to remember every old file.
                 */
                None => {
                    let mut f: PathBuf = path.parent().unwrap().to_path_buf();
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
    use crate::api::{ Cacheable, FileList, error::RequestError };
    use crate::config;
    use crate::local::update::UpdateChecker;
    use crate::test;
    use tokio::runtime::Runtime;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn file_has_update() -> Result<(), RequestError> {
        test::setup();
        let rt = Runtime::new().unwrap();

        let game: String = "morrowind".to_owned();
        let mod_id = 46599;

        let mut file_lists: HashMap<u32, FileList> = HashMap::new();

        let mut path = config::download_dir(&game);
        path.push("Graphic Herbalism MWSE - OpenMW-46599-1-03-1556986083.7z.json");
        println!("{:?}", path);

        let file_list = FileList::try_from_cache(&game, &mod_id)?;
        file_lists.insert(mod_id, file_list);

        let updater = UpdateChecker {
            game,
            updatable_mods: HashSet::new(),
            file_lists
        };

        assert_eq!(true, rt.block_on(updater.check_file(&path))?);
        Ok(())
    }
}
