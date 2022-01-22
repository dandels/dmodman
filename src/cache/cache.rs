use super::error::CacheError;
use super::PathType;
use super::{Cacheable, FileDetailsCache, FileListCache, LocalFile};
use crate::api::{DownloadLink, FileDetails, FileList};
use crate::config;

use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::sync::{Arc, RwLock};

use indexmap::IndexMap;
use tokio::fs;
use tokio::io;
use tokio_stream::StreamExt;

#[derive(Clone)]
pub struct Cache {
    pub game: Arc<String>,
    pub local_files: Arc<RwLock<Vec<LocalFile>>>,
    pub file_lists: FileListCache,
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
    pub async fn new(game: &str) -> Result<Self, CacheError> {
        let mut local_files: Vec<LocalFile> = Vec::new();
        let mut file_stream = fs::read_dir(config::download_dir(&game)).await?;
        while let Some(f) = file_stream.next_entry().await? {
            if f.path().is_file() && f.path().extension().and_then(OsStr::to_str) == Some("json") {
                local_files.push(LocalFile::from_path(&f.path()).await?);
            }
        }

        let mut file_lists: HashMap<(String, u32), FileList> = HashMap::new();
        let mut no_file_list_found: HashSet<u32> = HashSet::new();
        let mut file_details_map: IndexMap<u64, FileDetails> = IndexMap::new();

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
            match file_lists.get(&(f.game.clone(), f.mod_id)) {
                // found during previous iteration
                Some(fl) => file_list = fl.clone(),
                // not found during previous iteration, checking cache
                None => match FileList::try_from_cache(PathType::FileList(&game, &f.mod_id).path()).await {
                    Ok(fl) => {
                        file_list = fl.clone();
                        file_lists.insert((f.game.to_string(), f.mod_id), fl);
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
        }

        Ok(Self {
            game: Arc::new(game.to_owned()),
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

    pub async fn save_download_link(
        &self,
        dl: &DownloadLink,
        game: &str,
        mod_id: &u32,
        file_id: &u64,
    ) -> Result<(), CacheError> {
        let path = PathType::DownloadLink(game, mod_id, file_id).path();
        dl.save_to_cache(path).await?;
        Ok(())
    }

    pub async fn save_file_list(&self, fl: &FileList, game: &str, mod_id: &u32) -> Result<(), CacheError> {
        let path = PathType::FileList(&game, &mod_id).path();
        fl.save_to_cache(path).await?;
        self.file_lists.insert((game.to_string(), *mod_id), fl.clone());
        Ok(())
    }

    // TODO LocalFile's should use PathType. It's currently special cased since it's not an API response type.
    pub async fn save_local_file(&self, lf: LocalFile) -> Result<bool, io::Error> {
        // returns early if LocalFile already exists
        if self.local_files.read().unwrap().iter().any(|f| f.file_id == lf.file_id) {
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
    use super::CacheError;

    #[tokio::test]
    async fn load_cache() -> Result<(), CacheError> {
        let game = "morrowind";
        let cache = Cache::new(&game).await?;

        let _fd = cache.file_details.get(&82041).unwrap();
        Ok(())
    }
}
