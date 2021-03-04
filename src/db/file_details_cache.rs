use crate::api::FileDetails;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct FileDetailsCache {
    pub map: Arc<RwLock<HashMap<u64, FileDetails>>>,
    is_changed: bool, // used by UI to ask if file table needs to be redrawn
    len: usize,       // used by UI controls, so we only update when asking for is_changed
}

impl FileDetailsCache {
    pub fn new(map: HashMap<u64, FileDetails>) -> Self {
        let len = &map.keys().len();
        Self {
            map: Arc::new(RwLock::new(map)),
            is_changed: false,
            len: *len,
        }
    }

    pub fn insert(&mut self, key: u64, value: FileDetails) {
        self.map.write().unwrap().insert(key, value);
        self.is_changed = true;
        self.len = self.map.read().unwrap().keys().len()
    }

    pub fn remove(&mut self, key: &u64) {
        self.map.write().unwrap().remove(key);
        self.is_changed = true;
        self.len = self.map.read().unwrap().keys().len()
    }

    // this is a method for consistency with len() methods
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_changed(&mut self) -> bool {
        let ret = self.is_changed;
        self.is_changed = !self.is_changed;
        return ret;
    }
}
