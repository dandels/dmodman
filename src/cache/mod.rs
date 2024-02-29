pub mod cache_error;
mod cacheable;
mod file_data;
mod file_index;
mod file_lists;
mod local_file;
mod md5result_map;

pub use cache_error::CacheError;
pub use cacheable::Cacheable;
pub use file_data::FileData;
pub use file_index::*;
pub use file_lists::*;
pub use local_file::*;
pub use md5result_map::*;

use crate::api::Md5Results;
use crate::api::{DownloadLink, FileList};
use crate::config::{Config, DataType};
use crate::Logger;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::fs;
use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncWriteExt;

// The Cache is a basic file storage. There's a case to be made for using a relational database instead.
#[derive(Clone)]
pub struct Cache {
    config: Config,
    logger: Logger,
    pub file_lists: FileLists,
    pub file_index: FileIndex,
    pub md5result: Md5ResultMap,
    pub last_update_check: Arc<AtomicU64>,
}

impl Cache {
    pub async fn new(config: Config, logger: Logger) -> Result<Self, CacheError> {
        let file_lists = FileLists::new(config.clone()).await?;
        let md5result = Md5ResultMap::new(config.clone(), logger.clone());
        let file_index = FileIndex::new(config.clone(), logger.clone(), file_lists.clone(), md5result.clone()).await?;
        let last_update_check = load_last_updated(&config);

        Ok(Self {
            config,
            logger,
            file_lists,
            file_index,
            md5result,
            last_update_check,
        })
    }

    pub async fn save_download_links(
        &self,
        dl: &DownloadLink,
        game: &str,
        mod_id: u32,
        file_id: u64,
    ) -> Result<(), CacheError> {
        let path = self.config.path_for(DataType::DownloadLink(game, mod_id, file_id));
        dl.save(path).await?;
        Ok(())
    }

    /* FileLists can contain thousands of lines that we will never use
     * Delete records of files or updates that are older than what we have downloaded.
     * This saves space, memory, and a lot of time spent compressing. */
    pub async fn format_file_list(&self, fl: &mut FileList, game: &str, mod_id: u32) {
        /* These should be sorted but it's not guaranteed. The sort algorithm is O(n) if they are sorted.
         * The update checker also needs the file updates sorted. */
        fl.files.sort();
        fl.file_updates.sort();

        if let Some(files) = self.file_index.get_modfiles(&game.to_string(), &mod_id).await {
            let latest = files.peek().unwrap();
            let mut lowest_fileid = latest.file_id;
            let mut earliest_timestamp = latest.uploaded_timestamp().unwrap_or_default();

            for fd in files {
                if fd.file_id < lowest_fileid {
                    lowest_fileid = fd.file_id;
                }
                if earliest_timestamp < fd.uploaded_timestamp().unwrap() {
                    earliest_timestamp = fd.uploaded_timestamp().unwrap();
                }
            }

            if let Some(mut index) = fl.files.len().checked_sub(1) {
                while index > 0 {
                    if fl.files.get(index).unwrap().file_id > lowest_fileid {
                        index -= 1;
                    } else {
                        fl.files.drain(..index);
                        fl.files.shrink_to_fit();
                        break;
                    }
                }
            }
            if let Some(mut index) = fl.file_updates.len().checked_sub(1) {
                while index > 0 {
                    if fl.file_updates.get(index).unwrap().uploaded_timestamp > earliest_timestamp {
                        index -= 1;
                    } else {
                        fl.file_updates.drain(..index);
                        fl.file_updates.shrink_to_fit();
                        break;
                    }
                }
            }
        } else {
            self.logger.log("No local file to compare with");
        }
    }

    pub async fn save_file_list(&self, fl: &FileList, game: &str, mod_id: u32) -> Result<(), CacheError> {
        let path = self.config.path_for(DataType::FileList(game, mod_id));

        fl.save_compressed(path).await?;
        self.file_lists.insert((game, mod_id), fl.clone()).await;
        Ok(())
    }

    pub async fn save_local_file(&self, lf: LocalFile) -> Result<(), io::Error> {
        lf.save(self.config.path_for(DataType::LocalFile(&lf))).await?;
        self.file_index.add(lf).await;
        Ok(())
    }

    pub async fn save_md5result(&self, res: &Md5Results) {
        let game = &res.r#mod.domain_name;
        let path = self.config.path_for(DataType::Md5Results(game, res.file_details.file_id));
        if let Err(e) = res.save_compressed(path).await {
            self.logger.log(format!("Failed to save Md5Search to disk: {e}"));
        }
        self.md5result.insert(game.clone(), res.clone()).await;
    }

    pub async fn save_last_updated(&self, time: u64) -> Result<(), io::Error> {
        self.last_update_check.store(time, Ordering::Relaxed);
        let mut path = self.config.cache_for_profile();
        fs::create_dir_all(&path).await?;
        path.push("last_updated");
        let mut file = File::create(path).await?;
        file.write_all(format!("{}", time).as_bytes()).await
    }
}

// Loads timestamp from $XDG_CACHE_DIR/dmodman/$profile/last_updated
fn load_last_updated(config: &Config) -> Arc<AtomicU64> {
    let mut path = config.cache_for_profile();
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
    use crate::Logger;

    #[tokio::test]
    async fn load_file_details() -> Result<(), CacheError> {
        let profile = "morrowind";
        let config = ConfigBuilder::default().profile(profile).build().unwrap();
        let cache = Cache::new(config.clone(), Logger::default()).await?;

        let fdata = cache.file_index.get_by_file_id(&82041).await.unwrap();
        println!("{:?}", fdata);
        assert_eq!(fdata.local_file.game, profile);
        Ok(())
    }
}
