use super::DownloadStatus;

use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, RwLock,
};

#[derive(Clone, Default)]
pub struct Downloads {
    pub statuses: Arc<RwLock<Vec<Arc<RwLock<DownloadStatus>>>>>,
    is_changed: Arc<AtomicBool>, // used by UI to ask if error list needs to be redrawn
    len: Arc<AtomicUsize>,
}

impl Downloads {
    pub fn add(&mut self, status: Arc<RwLock<DownloadStatus>>) {
        self.statuses.write().unwrap().push(status);
        self.is_changed.store(true, Ordering::Relaxed);
        self.len.fetch_add(1, Ordering::Relaxed);
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.len.load(Ordering::Relaxed) == 0
    }
}
