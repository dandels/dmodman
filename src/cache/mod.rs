pub mod cache_error;
mod cacheable;
mod file_data;
mod file_index;
mod file_lists;
mod local_file;

pub use cache_error::CacheError;
pub use cacheable::Cacheable;
pub use file_data::FileData;
pub use file_index::*;
pub use file_lists::*;
pub use local_file::*;

use crate::api::{DownloadLink, FileList, Updated};
use crate::config::{Config, PathType};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::fs;
use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncWriteExt;

#[derive(Clone)]
pub struct Cache {
    pub file_lists: FileLists,
    pub file_index: FileIndex,
    config: Config,
    pub last_update_check: Arc<AtomicU64>,
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
        let file_lists = FileLists::new(&config).await?;
        let file_index = FileIndex::new(&config, file_lists.clone()).await?;

        Ok(Self {
            config: config.clone(),
            file_lists,
            file_index,
            last_update_check: load_last_updated(config),
        })
    }

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

    pub async fn save_last_updated(&self, time: u64) -> Result<(), io::Error> {
        self.last_update_check.store(time, Ordering::Relaxed);
        let mut path = self.config.cache_dir();
        fs::create_dir_all(&path).await?;
        path.push("last_updated");
        let mut file = File::create(path).await?;
        file.write_all(format!("{}", time).as_bytes()).await
    }

    // Delete a file and its metadata based on its index in file_index.files_sorted.
    pub async fn delete_by_index(&self, i: usize) -> Result<(), io::Error> {
        let mut fs_lock = self.file_index.files_sorted.write().await;
        let mut game_mods_lock = self.file_index.game_to_mods_map.write().await;
        let mut files_lock = self.file_index.file_id_map.write().await;
        let fd = fs_lock.get(i).unwrap().clone();
        let lf_lock = fd.local_file.write().await;
        let id_to_delete = fs_lock.get(i).unwrap().file_id;

        files_lock.remove(&id_to_delete);

        fs_lock.remove(i);

        let mods_map = game_mods_lock.get_mut(&lf_lock.game).unwrap();
        let heap = mods_map.get_mut(&lf_lock.mod_id).unwrap();
        heap.retain(|fdata| fdata.file_id != id_to_delete);
        if heap.is_empty() {
            mods_map.shift_remove(&lf_lock.mod_id);
        }
        if mods_map.is_empty() {
            game_mods_lock.remove(&lf_lock.game);
        }

        let mut path = self.config.path_for(PathType::LocalFile(&lf_lock));
        fs::remove_file(&path).await?;
        path.pop();
        path.push(&lf_lock.file_name);
        fs::remove_file(path).await?;
        self.file_index.has_changed.store(true, Ordering::Relaxed);
        Ok(())
    }
}

// Loads timestamp from $XDG_CACHE_DIR/dmodman/last_updated
fn load_last_updated(config: &Config) -> Arc<AtomicU64> {
    let mut path = config.cache_dir();
    path.push("last_updated");
    match std::fs::read_to_string(path) {
        Ok(contents) => AtomicU64::new(contents.parse::<u64>().unwrap_or_default()).into(),
        Err(_) => AtomicU64::new(0).into(),
    }
}

#[cfg(test)]
mod test {
    use super::Cache;
    use super::CacheError;
    use crate::config::ConfigBuilder;

    #[tokio::test]
    async fn load_file_details() -> Result<(), CacheError> {
        let profile = "morrowind";
        let config = ConfigBuilder::default().profile(profile).build().unwrap();
        let cache = Cache::new(&config).await?;

        let lock = cache.file_index.file_id_map.read().await;
        let fdata = lock.get(&82041).unwrap();
        println!("{:?}", fdata);
        assert_eq!(fdata.local_file.read().await.game, profile);
        Ok(())
    }
}
