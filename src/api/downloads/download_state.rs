use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

/* Serde can't serialize tokio's Rwlock.
 * We'll just use an AtomicU8 and convert it to an enum in the few places where it's needed. */

pub const DL_STATE_COMPLETE: u8 = 0;
pub const DL_STATE_DOWNLOADING: u8 = 1;
pub const DL_STATE_PAUSED: u8 = 2;
pub const DL_STATE_ERROR: u8 = 3;

#[derive(Debug, Deserialize, Serialize)]
pub enum DownloadState {
    Complete,
    Downloading,
    Error,
    Paused,
}

pub fn to_enum(num: Arc<AtomicU8>) -> DownloadState {
    match num.load(Ordering::Relaxed) {
        DL_STATE_COMPLETE => DownloadState::Complete,
        DL_STATE_DOWNLOADING => DownloadState::Downloading,
        DL_STATE_PAUSED => DownloadState::Paused,
        DL_STATE_ERROR | _ => DownloadState::Error,
    }
}
