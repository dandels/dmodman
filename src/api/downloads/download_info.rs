use super::download_state::*;
use super::FileInfo;
use serde::{Deserialize, Serialize};
use std::sync::{atomic::AtomicU8, Arc};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DownloadInfo {
    pub file_info: FileInfo,
    pub url: Url,
    pub state: Arc<AtomicU8>,
}

impl DownloadInfo {
    pub fn new(file_info: FileInfo, url: Url) -> Self {
        Self {
            file_info,
            url,
            state: Arc::new(DL_STATE_DOWNLOADING.into()),
        }
    }
}
