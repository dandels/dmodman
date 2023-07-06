use super::NxmUrl;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileInfo {
    pub game: String,
    pub mod_id: u32,
    pub file_id: u64,
    pub file_name: String,
}

impl FileInfo {
    pub fn from_nxm(nxm: &NxmUrl, file_name: String) -> Self {
        Self {
            game: nxm.domain_name.to_owned(),
            mod_id: nxm.mod_id,
            file_id: nxm.file_id,
            file_name,
        }
    }

    pub fn new(game: String, mod_id: u32, file_id: u64, file_name: String) -> Self {
        Self {
            game,
            mod_id,
            file_id,
            file_name,
        }
    }
}
