use crate::api::FileDetails;
use indexmap::IndexMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};

#[derive(Clone)]
pub struct FileDetailsCache {
    pub map: Arc<RwLock<IndexMap<u64, FileDetails>>>,
    is_changed: Arc<AtomicBool>, // used by UI to ask if file table needs to be redrawn
}

impl FileDetailsCache {
    pub fn new(map: IndexMap<u64, FileDetails>) -> Self {
        Self {
            map: Arc::new(RwLock::new(map)),
            is_changed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn insert(&self, key: u64, value: FileDetails) {
        self.map.try_write().unwrap().insert(key, value);
        self.is_changed.store(true, Ordering::Relaxed);
    }

    pub fn get(&self, key: &u64) -> Option<FileDetails> {
        match self.map.try_read().unwrap().get(key) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    pub fn get_index(&self, index: usize) -> Option<(u64, FileDetails)> {
        match self.map.try_read().unwrap().get_index(index) {
            Some((k, v)) => Some((k.clone(), v.clone())),
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

    pub fn is_changed(&self) -> bool {
        self.is_changed.load(Ordering::Relaxed)
    }
}
