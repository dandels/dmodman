use super::{CacheError, Cacheable, FileDetailsCache, FileListCache, LocalFile};
use crate::api::{DownloadLinks, FileDetails, FileList};
use crate::config::{Config, PathType};

use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use tokio::fs;
use tokio::io;
use tokio_stream::StreamExt;

#[derive(Clone)]
pub struct Cache {
    pub local_files: Arc<RwLock<Vec<LocalFile>>>,
    pub file_lists: FileListCache,
    pub file_details: FileDetailsCache,
    config: Config,
}

impl Cache {
    /* For each json file in downloads directory, serializes it to a LocalFile.
     * For each LocalFile, checks $cache/$mod/$mod_id.json for the FileList.
     *
     * Returns maps for:
     * - mod_id  -> FileList
     * - file_id -> FileDetails
     */
    pub async fn new(config: &Config) -> Result<Self, CacheError> {
        let mut local_files: Vec<LocalFile> = Vec::new();
        let mut file_lists: HashMap<(String, u32), FileList> = HashMap::new();
        let mut no_file_list_found: HashSet<u32> = HashSet::new();
        let mut file_details_map: IndexMap<u64, FileDetails> = IndexMap::new();

        let mut file_stream;
        match fs::read_dir(config.download_dir()).await {
            Ok(fs) => file_stream = fs,
            Err(_e) => {
                println!("Found no mod data, initializing empty cache.");
                return Ok(Self {
                    config: config.clone(),
                    local_files: Arc::new(RwLock::new(local_files)),
                    file_lists: FileListCache::new(file_lists),
                    file_details: FileDetailsCache::new(file_details_map),
                })
            }
        }
        while let Some(f) = file_stream.next_entry().await? {
            if f.path().is_file() && f.path().extension().and_then(OsStr::to_str) == Some("json") {
                local_files.push(LocalFile::from_path(&f.path()).await?);
            }
        }

        /* For each LocalFile, if that file's mod already has a FileList mapped, we use it. Otherwise we load it from
         * disk. It's possible that a LocalFile has no corresponding FileList (the API forgot about an old file or it's
         * a foreign file), so we wrap it in an option to remember if we already tried once to find it or not.
         */
        let mut lf_stream = tokio_stream::iter(&local_files);
        while let Some(f) = lf_stream.next().await {
            if no_file_list_found.contains(&f.mod_id) {
                continue;
            }

            let file_list: FileList;
            match file_lists.get(&(f.game.clone(), f.mod_id)) {
                // found during previous iteration
                Some(fl) => file_list = fl.clone(),
                // not found during previous iteration, checking cache
                None => {
                    let foo = config.path_for(PathType::FileList(&f.mod_id));
                    match FileList::try_from_cache(foo).await {
                        Ok(fl) => {
                            file_list = fl.clone();
                            file_lists.insert((f.game.to_string(), f.mod_id), fl);
                        }
                        Err(_e) => {
                            no_file_list_found.insert(f.mod_id);
                            continue;
                        }
                    }
                },
            }
            if let Some(fd) = file_list.files.iter().find(|fd| fd.file_id == f.file_id) {
                file_details_map.insert(f.file_id, fd.clone());
            }
        }

        Ok(Self {
            config: config.clone(),
            local_files: Arc::new(RwLock::new(local_files)),
            file_lists: FileListCache::new(file_lists),
            file_details: FileDetailsCache::new(file_details_map),
        })
    }

    /* TODO: when adding LocalFile,
     * - Check if FileDetails is required (probably yes)
     * - Send request for FileList if not present(?)
     * - Add the FileDetails to FileDetailsCache
     */

    pub async fn save_download_links(
        &self,
        dl: &DownloadLinks,
        mod_id: &u32,
        file_id: &u64,
    ) -> Result<(), CacheError> {
        let path = self.config.path_for(PathType::DownloadLinks(mod_id, file_id));
        dl.save_to_cache(path).await?;
        Ok(())
    }

    pub async fn save_file_list(&self, fl: &FileList, mod_id: &u32) -> Result<(), CacheError> {
        let path = self.config.path_for(PathType::FileList(&mod_id));
        fl.save_to_cache(path).await?;
        self.file_lists.insert((self.config.game.clone().unwrap().to_string(), *mod_id), fl.clone());
        Ok(())
    }

    // TODO LocalFile's should use PathType. It's currently special cased since it's not an API response type.
    pub async fn save_local_file(&self, lf: LocalFile) -> Result<bool, io::Error> {
        // returns early if LocalFile already exists
        if self.local_files.read().unwrap().iter().any(|f| f.file_id == lf.file_id) {
            return Ok(true);
        }

        lf.write(&self.config).await?;
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
    use super::CacheError;
    use crate::Config;

    #[tokio::test]
    async fn load_cache() -> Result<(), CacheError> {
        let game = "morrowind";
        let config = Config::new(Some(game), None).unwrap();
        let cache = Cache::new(&config).await?;

        let _fd = cache.file_details.get(&82041).unwrap();
        Ok(())
    }
}
