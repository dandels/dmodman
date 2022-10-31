use super::CacheError;
use super::{FileListCache, LocalFileCache};
use crate::api::FileDetails;
use indexmap::IndexMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::runtime::Handle;
use tokio::sync::RwLock;
use tokio::task;

// TODO handle foreign files
#[derive(Clone)]
pub struct FileIndex {
    map: Arc<RwLock<IndexMap<u64, FileDetails>>>,
    pub has_changed: Arc<AtomicBool>, // used by UI to ask if file table needs to be redrawn
}

/* TODO there's a lot of stuff here that would be nice to have in a trait, but traits can't currently access
 * fields.
 */
impl FileIndex {
    pub async fn new(local_files: &LocalFileCache, file_lists: &FileListCache) -> Result<Self, CacheError> {
        let mut map: IndexMap<u64, FileDetails> = IndexMap::new();

        for lf in local_files.items().await {
            if let Some(file_list) = file_lists.get(&lf.mod_id).await {
                if let Some(fd) = file_list.files.iter().find(|fd| fd.file_id == lf.file_id) {
                    map.insert(lf.file_id, fd.clone());
                }
            }
        }

        Ok(Self {
            map: Arc::new(RwLock::new(map)),
            has_changed: Arc::new(AtomicBool::new(false)),
        })
    }

    pub async fn insert(&self, key: u64, value: FileDetails) {
        self.map.write().await.insert(key, value);
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn get(&self, key: &u64) -> Option<FileDetails> {
        match self.map.read().await.get(key) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    pub async fn items(&self) -> Vec<FileDetails> {
        self.map.read().await.values().cloned().collect()
    }

    // TODO race condition in UI parts relying on this?
    pub fn len(&self) -> usize {
        /* This is annoying, but the traits for highlighting/selecting UI elements requires this function to not be
         * async */
        task::block_in_place(move || Handle::current().block_on(async move { self.map.read().await.len() }))
    }

    pub async fn get_index(&self, i: usize) -> Option<(u64, FileDetails)> {
        match self.map.read().await.get_index(i) {
            Some((k, v)) => Some((k.clone(), v.clone())),
            None => None,
        }
    }
}
