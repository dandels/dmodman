pub mod download_status;
pub mod nxm_url;
pub use self::download_status::*;
pub use self::nxm_url::*;

use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, RwLock,
};

#[derive(Clone, Default)]
pub struct Downloads {
    pub statuses: Arc<RwLock<Vec<Arc<RwLock<DownloadStatus>>>>>,
    has_changed: Arc<AtomicBool>, // used by UI to ask if error list needs to be redrawn
    len: Arc<AtomicUsize>,
}

impl Downloads {
    pub fn set_changed(&self) {
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub fn has_changed(&self) -> bool {
        let ret = self.has_changed.load(Ordering::Relaxed);
        self.has_changed.store(false, Ordering::Relaxed);
        ret
    }

    pub fn add(&self, status: Arc<RwLock<DownloadStatus>>) {
        self.statuses.write().unwrap().push(status);
        self.has_changed.store(true, Ordering::Relaxed);
        self.len.fetch_add(1, Ordering::Relaxed);
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len.load(Ordering::Relaxed) == 0
    }
}
