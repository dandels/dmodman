pub mod download_status;
pub mod nxm_url;
pub use self::download_status::*;
pub use self::nxm_url::*;
use indexmap::IndexMap;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct Downloads {
    pub statuses: Arc<RwLock<IndexMap<u64, DownloadStatus>>>,
    pub has_changed: Arc<AtomicBool>,
}

impl Downloads {
    pub async fn get(&self, file_id: &u64) -> Option<DownloadStatus> {
        self.statuses.read().await.get(file_id).cloned()
    }

    pub async fn add(&self, status: DownloadStatus) {
        self.statuses.write().await.insert(status.file_id, status);
        self.has_changed.store(true, Ordering::Relaxed);
    }
}
