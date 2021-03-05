use super::DownloadError;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Downloads {
    pub statuses: Arc<RwLock<Vec<Arc<RwLock<DownloadStatus>>>>>,
    len: usize,
}

impl Downloads {
    pub fn new() -> Self {
        Self {
            statuses: Arc::new(RwLock::new(Vec::new())),
            len: 0,
        }
    }

    pub fn add(&mut self, status: Arc<RwLock<DownloadStatus>>) {
        self.statuses.write().unwrap().push(status);
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

pub enum DownloadState {
    Downloading,
    Complete,
    Failed(DownloadError),
}

pub struct DownloadStatus {
    pub file_name: String,
    pub file_id: u64,
    bytes_read: u64,
    pub bytes_total: Option<u64>,
    pub state: DownloadState,
}

impl DownloadStatus {
    pub fn new(file_name: String, file_id: u64) -> Self {
        Self {
            file_name,
            file_id,
            bytes_read: 0,
            bytes_total: None,
            state: DownloadState::Downloading,
        }
    }

    /* TODO use some kind of task/thread parking to pause/continue downloads?:
     * - https://tokio-rs.github.io/tokio/doc/tokio/sync/struct.Notify.html
     * - https://doc.rust-lang.org/std/thread/fn.park.html
     *
     * The HTTP range header might be required for cold-resuming downloads, which might also mean we don't need to park
     * threads. The easiest is probably simply stopping downloads and calculating their size in bytes when resuming.
     */

    pub fn update_progres(&mut self, bytes: u64) {
        self.bytes_read += bytes;
    }

    pub fn progress(&self) -> String {
        match self.bytes_total {
            Some(total) => (100 * (self.bytes_read / total)).to_string(),
            None => "NIL".to_string(),
        }
    }
}
