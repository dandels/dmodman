use super::DownloadProgress;
use crate::cache::Cacheable;
use super::FileInfo;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use url::Url;

const DL_STATE_DONE: u8 = 0;
const DL_STATE_DOWNLOADING: u8 = 1;
const DL_STATE_ERROR: u8 = 2;
const DL_STATE_EXPIRED: u8 = 3;
const DL_STATE_PAUSED: u8 = 4;

/* Serde can't serialize tokio's Rwlock.
 * We'll just use an AtomicU8 and convert it to an enum in the few places where it's needed.
 * Serializing the state allows us to restore the download state on startup. */
#[derive(Debug, Deserialize, Serialize)]
pub enum DownloadState {
    Done,
    Downloading,
    Error,
    Expired,
    Paused,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct DownloadInfo {
    pub file_info: FileInfo,
    pub url: Url,
    state: Arc<AtomicU8>,
    pub progress: DownloadProgress,
}

impl DownloadInfo {
    pub fn new(file_info: FileInfo, url: Url) -> Self {
        Self {
            file_info,
            url,
            state: Arc::new(DL_STATE_DOWNLOADING.into()),
            progress: DownloadProgress::default(),
        }
    }

    pub fn set_state(&self, state_enum: DownloadState) {
        self.state.store(
            match state_enum {
                DownloadState::Done => DL_STATE_DONE,
                DownloadState::Downloading => DL_STATE_DOWNLOADING,
                DownloadState::Error => DL_STATE_ERROR,
                DownloadState::Expired => DL_STATE_EXPIRED,
                DownloadState::Paused => DL_STATE_PAUSED,
            },
            Ordering::Relaxed,
        );
    }

    pub fn get_state(&self) -> DownloadState {
        match self.state.load(Ordering::Relaxed) {
            DL_STATE_DONE => DownloadState::Done,
            DL_STATE_DOWNLOADING => DownloadState::Downloading,
            DL_STATE_ERROR => DownloadState::Error,
            DL_STATE_PAUSED => DownloadState::Paused,
            // Treat any other value as expired because the user has to restart the download anyway.
            _ => DownloadState::Expired,
        }
    }
}

impl fmt::Display for DownloadState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DownloadState::Done => write!(f, "Done"),
            DownloadState::Error => write!(f, "Error"),
            DownloadState::Expired => write!(f, "Expired"),
            DownloadState::Downloading => write!(f, "Downloading"),
            DownloadState::Paused => write!(f, "Paused"),
        }
    }
}

impl Cacheable for DownloadInfo {}
