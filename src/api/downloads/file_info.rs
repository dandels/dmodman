use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileInfo {
    pub game: String,
    pub mod_id: u32,
    pub file_id: u64,
    pub file_name: String,
}

impl FileInfo {
    pub fn new(game: String, mod_id: u32, file_id: u64, file_name: String) -> Self {
        Self {
            game,
            mod_id,
            file_id,
            file_name,
        }
    }
}
