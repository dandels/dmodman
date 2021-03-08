use crate::api::{FileDetails, FileList};
use crate::config;
use super::error::DbError;
use super::{Cacheable, FileDetailsCache, LocalFile};

use tokio_stream::StreamExt;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use tokio::fs;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Cache {
    pub game: Arc<String>,
    pub local_files: Arc<RwLock<Vec<LocalFile>>>,
    pub file_list_map: Arc<RwLock<HashMap<(String, u32), FileList>>>,
    pub file_details: FileDetailsCache,
}

impl Cache {
    /* For each json file in downloads directory, serializes it to a LocalFile.
     * For each LocalFile, checks $cache/$mod/$mod_id.json for the FileList.
     *
     * Returns maps for:
     * - mod_id  -> FileList
     * - file_id -> FileDetails
     */
    pub async fn new(game: &str) -> Result<Self, DbError> {
        let mut local_files: Vec<LocalFile> = Vec::new();
        let mut file_stream = fs::read_dir(config::download_dir(&game)).await?;
        while let Some(f) = file_stream.next_entry().await? {
            if f.path().is_file() && f.path().extension().and_then(OsStr::to_str) == Some("json") {
                local_files.push(LocalFile::from_path(&f.path()).await?);
            }
        };

        let mut file_list_map: HashMap<(String, u32), FileList> = HashMap::new();
        let mut no_file_list_found: HashSet<u32> = HashSet::new();
        let mut file_details_map: HashMap<u64, FileDetails> = HashMap::new();

        /* For each LocalFile, if that file's mod already has a FileList mapped, we use it. Otherwise we load it from
         * disk. It's possible that a LocalFile has no corresponding FileList (the API forgot about an old file or it's
         * a foreign file), so we wrap it in an option to remember if we already tried once to find it or not.
         */
        let mut errors: Vec<String> = Vec::new();

        let mut lf_stream = tokio_stream::iter(&local_files);
        while let Some(f) = lf_stream.next().await {
            if no_file_list_found.contains(&f.mod_id) {
                continue;
            }

            let file_list: FileList;
            match file_list_map.get(&(f.game.clone(), f.mod_id)) {
                // found during previous iteration
                Some(fl) => file_list = fl.clone(),
                // not found during previous iteration, checking cache
                None => match FileList::try_from_cache(&game, &f.mod_id).await {
                    Ok(fl) => {
                        file_list = fl.clone();
                        file_list_map.insert((f.game.clone(), f.mod_id), fl);
                    }
                    Err(e) => {
                        errors.append(&mut vec![e.to_string()]);
                        no_file_list_found.insert(f.mod_id);
                        continue;
                    }
                },
            }
            if let Some(fd) = file_list.files.iter().find(|fd| fd.file_id == f.file_id) {
                file_details_map.insert(f.file_id, fd.clone());
            }
        };

        Ok(Self {
            game: Arc::new(game.to_owned()),
            local_files: Arc::new(RwLock::new(local_files)),
            file_list_map: Arc::new(RwLock::new(file_list_map)),
            file_details: FileDetailsCache::new(file_details_map),
        })
    }

    pub async fn save_file_list(&self, game: &str, fl: FileList, mod_id: &u32) -> Result<(), DbError> {
        fl.save_to_cache(&game, mod_id).await?;
        self.file_list_map
            .write()
            .unwrap()
            .insert((game.to_string(), *mod_id), fl);
        Ok(())
    }

    /* returns whether FileDetails is up to date
     * TODO figure out how to keep the cache file types in sync with eachother
     */
    pub async fn save_local_file(&self, lf: LocalFile) -> Result<bool, DbError> {
        let is_present: bool = self.local_files.read().unwrap().iter().any(|f| f.file_id == lf.file_id);

        if is_present {
            return Ok(true);
        }

        lf.write().await?;
        let file_id = lf.file_id;
        self.local_files.write().unwrap().push(lf);

        match self.file_details.get(&file_id) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Cache;
    use super::DbError;

    #[tokio::test]
    async fn load_cache() -> Result<(), DbError> {
        let game = "morrowind";
        let cache = Cache::new(&game).await?;

        let _fd = cache.file_details.get(&82041).unwrap();
        Ok(())
    }
}
