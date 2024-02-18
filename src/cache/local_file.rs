use crate::api::downloads::FileInfo;
use crate::cache::Cacheable;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocalFile {
    pub game: String,
    pub file_name: String,
    pub mod_id: u32,
    pub file_id: u64,
    pub update_status: UpdateStatus,
}

impl LocalFile {
    pub fn new(fi: FileInfo, update_status: UpdateStatus) -> Self {
        LocalFile {
            game: fi.game,
            file_name: fi.file_name,
            mod_id: fi.mod_id,
            file_id: fi.file_id,
            update_status,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum UpdateStatus {
    UpToDate(u64),     // time of user's newest file,
    HasNewFile(u64),   // time of user's newest file
    OutOfDate(u64),    // time of user's newest file
    IgnoredUntil(u64), // time of latest file in update list
}

impl Cacheable for LocalFile {}

impl UpdateStatus {
    pub fn time(&self) -> u64 {
        match self {
            Self::UpToDate(t) | Self::HasNewFile(t) | Self::OutOfDate(t) | Self::IgnoredUntil(t) => *t,
        }
    }
}
