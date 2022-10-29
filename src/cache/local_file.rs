use crate::api::NxmUrl;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocalFile {
    pub game: String,
    pub file_name: String,
    pub mod_id: u32,
    pub file_id: u64,
}

impl LocalFile {
    pub fn new(nxm: &NxmUrl, file_name: String) -> Self {
        LocalFile {
            game: nxm.domain_name.to_owned(),
            file_name,
            mod_id: nxm.mod_id,
            file_id: nxm.file_id,
        }
    }
}
