use super::error::DbError;
use super::{Cacheable, LocalFile};
use crate::api::{FileDetails, FileList};
use crate::config;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;

pub struct Cache {
    pub game: String,
    pub local_files: Vec<LocalFile>,
    pub file_list_map: HashMap<u32, FileList>,
    pub file_details_map: HashMap<u64, FileDetails>,
}

impl Cache {
    /* For each json file in downloads directory, serializes it to a LocalFile.
     * For each LocalFile, checks $cache/$mod/$mod_id.json for the FileList.
     *
     * Returns maps for:
     * - mod_id  -> FileList
     * - file_id -> FileDetails
     */
    pub fn new(game: &str) -> Result<Self, DbError> {
        let local_files: Vec<LocalFile> = fs::read_dir(config::download_dir(&game))?
            .flatten()
            .filter_map(|x| {
                if x.path().is_file()
                    && x.path().extension().and_then(OsStr::to_str) == Some("json")
                {
                    Some(LocalFile::from_path(&x.path()).unwrap())
                } else {
                    None
                }
            })
            .collect();

        let mut file_list_map: HashMap<u32, FileList> = HashMap::new();
        let mut no_file_list_found: HashSet<u32> = HashSet::new();
        let mut file_details_map: HashMap<u64, FileDetails> = HashMap::new();

        /* For each LocalFile, if that file's mod already has a FileList mapped, we use it. Otherwise we fetch it.
         * It could be possible that a LocalFile has no corresponding FileList (the API forgot about an old file or it's
         * a foreign file), so we wrap it in an option to remember if we already tried once to find it or not.
         */
        let mut errors: Vec<String> = Vec::new();
        local_files.iter().for_each(|f| {
            if no_file_list_found.contains(&f.mod_id) {
                return;
            }

            let file_list: FileList;
            match file_list_map.get(&f.mod_id) {
                // found during previous iteration
                Some(fl) => file_list = fl.clone(),
                // not found during previous iteration, checking cache
                None => match FileList::try_from_cache(&game, &f.mod_id) {
                    Ok(fl) => {
                        file_list = fl.clone();
                        file_list_map.insert(f.mod_id, fl);
                    }
                    Err(e) => {
                        errors.append(&mut vec![e.to_string()]);
                        no_file_list_found.insert(f.mod_id);
                        return;
                    }
                },
            }
            if let Some(fd) = file_list.files.iter().find(|fd| fd.file_id == f.file_id) {
                file_details_map.insert(f.file_id, fd.clone());
            }
        });

        Ok(Self {
            game: game.to_owned(),
            local_files,
            file_list_map,
            file_details_map,
        })
    }

    pub fn save_file_list(&mut self, fl: FileList, mod_id: &u32) -> Result<(), std::io::Error> {
        fl.save_to_cache(&self.game, mod_id)?;
        self.file_list_map.insert(*mod_id, fl);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Cache;
    use super::DbError;

    // TODO verify results
    #[test]
    fn load_cache() -> Result<(), DbError> {
        let game = "morrowind";
        let cache = Cache::new(&game)?;
        println!("local files {:?}", cache.local_files);
        println!("file list map {:?}", cache.file_list_map);
        println!("file details map {:?}", cache.file_details_map);
        Ok(())
    }
}
