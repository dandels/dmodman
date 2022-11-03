use crate::api::NxmUrl;
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
    pub fn new(nxm: &NxmUrl, file_name: String, update_status: UpdateStatus) -> Self {
        LocalFile {
            game: nxm.domain_name.to_owned(),
            file_name,
            mod_id: nxm.mod_id,
            file_id: nxm.file_id,
            update_status,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum UpdateStatus {
    UpToDate(u64),
    HasNewFile(u64),
    OutOfDate(u64),
    IgnoredUntil(u64),
}

impl UpdateStatus {
    pub fn time(&self) -> u64 {
        match *self {
            UpdateStatus::UpToDate(t)
            | UpdateStatus::HasNewFile(t)
            | UpdateStatus::OutOfDate(t)
            | UpdateStatus::IgnoredUntil(t) => t,
        }
    }
}
