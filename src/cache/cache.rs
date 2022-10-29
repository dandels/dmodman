use super::{CacheError, Cacheable, FileDetailsCache, FileListCache, LocalFile, LocalFileCache};
use crate::api::{DownloadLinks, FileDetails, FileList};
use crate::config::{Config, PathType};

use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;

use indexmap::IndexMap;
use tokio::fs;
use tokio::io;
use tokio_stream::StreamExt;

#[derive(Clone)]
pub struct Cache {
    pub local_files: LocalFileCache,
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
        let mut local_files: HashMap<u64, LocalFile> = HashMap::new();
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
                    local_files: LocalFileCache::new(local_files),
                    file_lists: FileListCache::new(file_lists),
                    file_details: FileDetailsCache::new(file_details_map),
                })
            }
        }
        while let Some(f) = file_stream.next_entry().await? {
            if f.path().is_file() && f.path().extension().and_then(OsStr::to_str) == Some("json") {
                let lf = LocalFile::load(f.path()).await?;
                local_files.insert(lf.file_id, lf);
            }
        }

        /* For each LocalFile, if that file's mod already has a FileList mapped, we use it. Otherwise we load it from
         * disk. It's possible that a LocalFile has no corresponding FileList (the API forgot about an old file or it's
         * a foreign file), so we wrap it in an option to remember if we already tried once to find it or not.
         */
        let mut lf_stream = tokio_stream::iter(local_files.clone().into_values().collect::<Vec<LocalFile>>());
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
                    let fl = config.path_for(PathType::FileList(&f.mod_id));
                    match FileList::load(fl).await {
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
            local_files: LocalFileCache::new(local_files),
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
        dl.save(path).await?;
        Ok(())
    }

    pub async fn save_file_list(&self, fl: &FileList, mod_id: &u32) -> Result<(), CacheError> {
        let path = self.config.path_for(PathType::FileList(&mod_id));
        fl.save(path).await?;
        self.file_lists.insert((self.config.game().unwrap(), *mod_id), fl.clone());
        Ok(())
    }

    pub async fn add_local_file(&self, lf: LocalFile) -> Result<(), io::Error> {
        // returns early if LocalFile already exists
        if self.local_files.get(lf.file_id).is_some() {
            return Ok(());
        }

        lf.save(self.config.path_for(PathType::LocalFile(&lf))).await?;
        let file_id = lf.file_id;
        self.local_files.push(lf);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Cache;
    use super::CacheError;
    use crate::Config;
    use crate::InitialConfig;

    #[tokio::test]
    async fn load_cache() -> Result<(), CacheError> {
        let game = "morrowind";
        let config = Config::new(InitialConfig::default(), Some(game), None).unwrap();
        let cache = Cache::new(&config).await?;

        let _fd = cache.file_details.get(&82041).unwrap();
        Ok(())
    }
}
