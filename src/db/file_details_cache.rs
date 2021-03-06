use crate::api::FileDetails;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, RwLock,
};

#[derive(Clone)]
pub struct FileDetailsCache {
    pub map: Arc<RwLock<HashMap<u64, FileDetails>>>,
    is_changed: Arc<AtomicBool>, // used by UI to ask if file table needs to be redrawn
    len: Arc<AtomicUsize>, // used by UI controls, so we only update when asking for is_changed
}

impl FileDetailsCache {
    pub fn new(map: HashMap<u64, FileDetails>) -> Self {
        let len = &map.keys().len();
        Self {
            map: Arc::new(RwLock::new(map)),
            is_changed: Arc::new(AtomicBool::new(false)),
            len: Arc::new(AtomicUsize::new(*len)),
        }
    }

    pub fn insert(&self, key: u64, value: FileDetails) {
        self.map.write().unwrap().insert(key, value);
        self.is_changed.store(true, Ordering::Relaxed);
        self.len
            .store(self.map.read().unwrap().keys().len(), Ordering::Relaxed)
    }

    pub fn remove(&self, key: &u64) {
        self.map.write().unwrap().remove(key);
        self.is_changed.store(true, Ordering::Relaxed);
        self.len
            .store(self.map.read().unwrap().keys().len(), Ordering::Relaxed)
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    pub fn is_changed(&self) -> bool {
        let ret = self.is_changed.load(Ordering::Relaxed);
        self.is_changed
            .store(!self.is_changed.load(Ordering::Relaxed), Ordering::Relaxed);
        ret
    }
}
