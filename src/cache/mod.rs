mod archive_files;
pub mod cache_error;
mod cacheable;
mod file_lists;
mod installed;
mod md5result_map;
mod metadata_index;
mod modfile_metadata;

pub use archive_files::*;
pub use cache_error::CacheError;
pub use cacheable::Cacheable;
pub use file_lists::*;
pub use installed::*;
pub use md5result_map::*;
pub use metadata_index::*;
pub use modfile_metadata::ModFileMetadata;

use crate::api::Md5Results;
use crate::api::{DownloadLink, FileList};
use crate::config::{Config, DataPath};
use crate::Logger;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::fs;
use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncWriteExt;

#[derive(Clone)]
pub struct Cache {
    config: Config,
    logger: Logger,
    pub archives: ArchiveFiles,
    pub file_lists: FileLists,
    pub metadata_index: MetadataIndex,
    pub md5result: Md5ResultMap,
    pub last_update_check: Arc<AtomicU64>,
    pub installed: Installed,
}

impl Cache {
    pub async fn new(config: Config, logger: Logger) -> Result<Self, CacheError> {
        let file_lists = FileLists::new(config.clone()).await?;
        let md5result = Md5ResultMap::new(config.clone(), logger.clone());
        let metadata_index =
            MetadataIndex::new(config.clone(), logger.clone(), file_lists.clone(), md5result.clone()).await;
        let installed = Installed::new(config.clone(), logger.clone(), metadata_index.clone()).await;
        let archives =
            ArchiveFiles::new(config.clone(), logger.clone(), installed.clone(), metadata_index.clone()).await;
        let last_update_check = load_last_updated(&config);

        Ok(Self {
            archives,
            installed,
            config,
            logger,
            file_lists,
            metadata_index,
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
        let path = DataPath::DownloadLink(&self.config, game, mod_id, file_id).into();
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

        if let Some(files) = self.metadata_index.get_modfiles(&game.to_string(), &mod_id).await {
            let old_files_start = fl.files.partition_point(|f| f.file_id < files.first().unwrap().file_id);
            fl.files.drain(..old_files_start);
            fl.files.shrink_to_fit();

            let mut earliest_timestamp = u64::MAX;
            for mfd in files {
                let mut fd_lock = mfd.file_details.write().await;
                if fd_lock.is_none() {
                    if let Ok(i) = fl.files.binary_search_by(|f| f.file_id.cmp(&mfd.file_id)) {
                        *fd_lock = Some(fl.files.get(i).unwrap().clone());
                    }
                }

                match fd_lock.as_ref() {
                    Some(fd) => {
                        if earliest_timestamp < fd.uploaded_timestamp {
                            earliest_timestamp = fd.uploaded_timestamp;
                        }
                    }
                    None => {
                        let mut md5res_lock = mfd.md5results.write().await;
                        if let Some(res) = self.md5result.get(game, mfd.file_id).await {
                            *md5res_lock = Some(res.clone());
                            if earliest_timestamp < res.file_details.uploaded_timestamp {
                                earliest_timestamp = res.file_details.uploaded_timestamp;
                            }
                        }
                    }
                }
            }

            let old_updates_start = fl.file_updates.partition_point(|u| u.uploaded_timestamp < earliest_timestamp);
            fl.file_updates.drain(..old_updates_start);
            fl.file_updates.shrink_to_fit();
        } else {
            self.logger.log("No local file to compare with");
        }
    }

    pub async fn save_file_list(&self, fl: Arc<FileList>, game: &str, mod_id: u32) -> Result<(), CacheError> {
        let path = DataPath::FileList(&self.config, game, mod_id).into();

        fl.save_compressed(path).await?;
        self.file_lists.insert((game, mod_id), fl).await;
        Ok(())
    }

    pub async fn save_md5result(&self, res: &Md5Results) {
        let game = &res.r#mod.domain_name;
        let path = DataPath::Md5Results(&self.config, game, res.file_details.file_id).into();
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
        let profile = "testprofile";
        let file_id = 82041;
        let config = ConfigBuilder::default().profile(profile).build().unwrap();
        let cache = Cache::new(config.clone(), Logger::default()).await?;

        let fdata = cache.metadata_index.get_by_file_id(&file_id).await.unwrap();
        assert_eq!(fdata.file_id, file_id);
        Ok(())
    }
}
