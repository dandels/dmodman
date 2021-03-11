use crate::api::FileList;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct FileListCache {
    pub map: Arc<RwLock<HashMap<(String, u32), FileList>>>,
}

impl FileListCache {
    pub fn new(map: HashMap<(String, u32), FileList>) -> Self {
        Self {
            map: Arc::new(RwLock::new(map)),
        }
    }

    pub fn insert(&self, key: (String, u32), value: FileList) {
        self.map.try_write().unwrap().insert(key, value);
    }

    pub fn get(&self, game: &str, key: u32) -> Option<FileList> {
        match self.map.try_read().unwrap().get(&(game.to_string(), key)) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
}
