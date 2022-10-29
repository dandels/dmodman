use super::LocalFile;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct LocalFileCache {
    // This could be Vec instead of IndexMap, but we'll sometimes be querying stuff by file id
    map: Arc<RwLock<HashMap<u64, LocalFile>>>,
}

impl LocalFileCache {
    pub fn new(local_files: HashMap<u64, LocalFile>) -> Self {
        Self {
            map: Arc::new(RwLock::new(local_files)),
        }
    }

    pub fn push(&self, value: LocalFile) {
        self.map.try_write().unwrap().insert(value.file_id, value);
    }

    pub fn get(&self, key: u64) -> Option<LocalFile> {
        match self.map.try_read().unwrap().get(&key) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    pub fn items(&self) -> Vec<LocalFile> {
        self.map.try_read().unwrap().values().cloned().collect()
    }
}
