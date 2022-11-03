use super::FileListCache;
use super::{CacheError, Cacheable, LocalFile};
use crate::api::FileDetails;
use crate::config::Config;

use std::ffi::OsStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use indexmap::IndexMap;
use tokio::fs;
use tokio::runtime::Handle;
use tokio::sync::RwLock;
use tokio::task;

// TODO handle foreign files
#[derive(Clone)]
pub struct FileIndex {
    pub map: Arc<RwLock<IndexMap<u64, (LocalFile, Option<FileDetails>)>>>,
    pub has_changed: Arc<AtomicBool>, // used by UI to ask if file table needs to be redrawn
    file_lists: FileListCache,
}

impl FileIndex {
    pub async fn new(config: &Config, file_lists: FileListCache) -> Result<Self, CacheError> {
        // is it even possible for FileDetails to be missing?
        let mut map: IndexMap<u64, (LocalFile, Option<FileDetails>)> = IndexMap::new();

        /* This iterates through all files in the download directory for the current game. It serializes all json files
         * into LocalFiles, then checks in the FileList cache if there exists a corresponding FileDetails for that
         * LocalFile.
         */
        if let Ok(mut file_stream) = fs::read_dir(config.download_dir()).await {
            while let Some(f) = file_stream.next_entry().await? {
                // TODO don't assume file exists because json file exists
                if f.path().is_file() && f.path().extension().and_then(OsStr::to_str) == Some("json") {
                    if let Ok(lf) = LocalFile::load(f.path()).await {
                        if let Some(file_list) = file_lists.get((&lf.game, lf.mod_id)).await {
                            let fd = file_list.files.iter().find(|fd| fd.file_id == lf.file_id);
                            map.insert(lf.file_id, (lf, fd.cloned()));
                        }
                    }
                }
            }
        }

        Ok(Self {
            map: Arc::new(RwLock::new(map)),
            has_changed: Arc::new(AtomicBool::new(false)),
            file_lists,
        })
    }

    async fn get_filedetails(&self, local_file: &LocalFile) -> Option<FileDetails> {
        self.file_lists
            .get((&local_file.game, local_file.mod_id))
            .await
            .and_then(|list| list.files.iter().cloned().find(|fd| fd.file_id == local_file.file_id))
    }

    pub async fn add(&self, local_file: LocalFile) {
        let fd = self.get_filedetails(&local_file).await;
        self.map.write().await.insert(local_file.file_id, (local_file, fd));
        self.has_changed.store(true, Ordering::Relaxed);
    }

    // TODO race condition in UI parts relying on this?
    // FIXME
    pub fn len(&self) -> usize {
        /* This is annoying, but the traits for highlighting/selecting UI elements requires this function to not be
         * async */
        task::block_in_place(move || Handle::current().block_on(async move { self.map.read().await.len() }))
    }
}
