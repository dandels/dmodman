mod cache_error;
mod cacheable;
mod file_data;
mod file_index;
mod file_lists;
mod local_file;
pub use cache_error::*;
pub use cacheable::*;
pub use file_data::FileData;
pub use file_index::*;
pub use file_lists::*;
pub use local_file::*;

//use self::{CacheError, Cacheable, FileIndex, FileListCache, LocalFile};
use crate::api::{DownloadLink, FileList};
use crate::config::{Config, PathType};

use tokio::fs;
use tokio::io;

use std::sync::atomic::Ordering;

#[derive(Clone)]
pub struct Cache {
    pub file_lists: FileLists,
    pub file_index: FileIndex,
    config: Config,
}

impl Cache {
    /* For each json file in downloads directory, serializes it to a LocalFile.
     * For each LocalFile, checks $cache/$mod/$mod_id.json for the FileList.
     *
     * Creates maps for:
     * - file_id        -> LocalFile
     * - (game, mod_id) -> FileList
     * - file_id        -> FileDetails
     */
    pub async fn new(config: &Config) -> Result<Self, CacheError> {
        let file_lists = FileLists::new(config).await?;
        let file_index = FileIndex::new(config, file_lists.clone()).await?;

        Ok(Self {
            config: config.clone(),
            file_lists,
            file_index,
        })
    }

    /* TODO: when adding LocalFile,
     * - Check if FileDetails is required (probably yes)
     * - Send request for FileList if not present(?)
     * - Add the FileDetails to FileDetailsCache
     */
    pub async fn save_download_links(
        &self,
        dl: &DownloadLink,
        game: &str,
        mod_id: &u32,
        file_id: &u64,
    ) -> Result<(), CacheError> {
        let path = self.config.path_for(PathType::DownloadLink(game, mod_id, file_id));
        dl.save(path).await?;
        Ok(())
    }

    pub async fn save_file_list(&self, fl: &FileList, game: &str, mod_id: u32) -> Result<(), CacheError> {
        let path = self.config.path_for(PathType::FileList(game, &mod_id));
        fl.save(path).await?;
        self.file_lists.insert((game, mod_id), fl.clone()).await;
        Ok(())
    }

    pub async fn save_local_file(&self, lf: LocalFile) -> Result<(), io::Error> {
        lf.save(self.config.path_for(PathType::LocalFile(&lf))).await?;
        self.file_index.add(lf).await;
        Ok(())
    }

    pub async fn delete_by_index(&self, i: usize) -> Result<(), io::Error> {
        let mut fs_lock = self.file_index.files_sorted.write().await;
        let mut mf_lock = self.file_index.mod_file_map.write().await;
        let mut files_lock = self.file_index.file_id_map.write().await;
        let fd = fs_lock.get(i).unwrap().clone();
        let lf_lock = fd.local_file.write().await;
        mf_lock.remove(&(lf_lock.game.clone(), lf_lock.mod_id));
        fs_lock.remove(i);
        files_lock.remove(&fd.file_id);
        let mut path = self.config.path_for(PathType::LocalFile(&lf_lock));
        fs::remove_file(&path).await?;
        path.pop();
        path.push(&lf_lock.file_name);
        fs::remove_file(path).await?;
        self.file_index.has_changed.store(true, Ordering::Relaxed);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Cache;
    use super::CacheError;
    use crate::config::ConfigBuilder;

    #[tokio::test]
    async fn load_file_details() -> Result<(), CacheError> {
        let game = "morrowind";
        let config = ConfigBuilder::default().profile(game).build().unwrap();
        let cache = Cache::new(&config).await?;

        let lock = cache.file_index.file_id_map.read().await;
        let fdata = lock.get(&82041).unwrap();
        println!("{:?}", fdata);
        assert_eq!(fdata.local_file.read().await.game, game);
        Ok(())
    }
}
