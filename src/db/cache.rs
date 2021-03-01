use super::error::DbError;
use super::LocalFile;
use crate::api::{Cacheable, FileDetails, FileList};
use crate::config;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;

pub struct Cache {
    pub game: String,
    pub local_files: Vec<LocalFile>,
    pub file_list_map: HashMap<u32, Option<FileList>>,
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

        //
        let file_list_map: HashMap<u32, Option<FileList>> = HashMap::new();
        let file_details_map: HashMap<u64, FileDetails> = HashMap::new();
        local_files
            .iter()
            .for_each(|f| match file_list_map.get(&f.mod_id) {
                Some(fl_opt) => {
                    if let Some(fl) = fl_opt {
                        if let Some(fd) = fl.files.iter().find(|fd| fd.file_id == f.file_id) {
                            file_details_map.insert(f.file_id, *fd);
                        }
                    }
                }
                None => match FileList::try_from_cache(&game, &f.mod_id) {
                    Ok(fl) => file_list_map.insert(f.mod_id, Some(fl)),
                    Err(_e) => {
                        println!("Couldn't find FileList for LocalFile: {}", f.file_name);
                    }
                },
            });

        Ok(Self {
            game: game.to_owned(),
            local_files,
            file_list_map,
            file_details_map,
        })
    }
}
