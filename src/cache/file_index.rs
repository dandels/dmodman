use super::CacheError;
use super::{FileListCache, LocalFileCache};
use crate::api::FileDetails;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

// TODO handle foreign files
#[derive(Clone)]
pub struct FileIndex {
    map: Arc<RwLock<IndexMap<u64, FileDetails>>>,
    has_changed: Arc<AtomicBool>, // used by UI to ask if file table needs to be redrawn
}

/* TODO there's a lot of stuff here that would be nice to have in a trait, but traits can't currently access
 * fields.
 */
impl FileIndex {
    pub async fn new(local_files: &LocalFileCache, file_lists: &FileListCache) -> Result<Self, CacheError> {
        let mut map: IndexMap<u64, FileDetails> = IndexMap::new();

        for lf in local_files.items() {
            if let Some(file_list) = file_lists.get(&lf.mod_id) {
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

    pub fn insert(&self, key: u64, value: FileDetails) {
        self.map.try_write().unwrap().insert(key, value);
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub fn get(&self, key: &u64) -> Option<FileDetails> {
        match self.map.try_read().unwrap().get(key) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    pub fn items(&self) -> Vec<FileDetails> {
        self.map.try_read().unwrap().values().cloned().collect()
    }

    // TODO race condition in UI parts relying on this
    pub fn len(&self) -> usize {
        self.map.try_read().unwrap().keys().len()
    }

    pub fn has_changed(&self) -> bool {
        self.has_changed.load(Ordering::Relaxed)
    }

    pub fn get_index(&self, i: usize) -> Option<(u64, FileDetails)> {
        match self.map.try_read().unwrap().get_index(i) {
            Some((k, v)) => Some((k.clone(), v.clone())),
            None => None,
        }
    }
}
