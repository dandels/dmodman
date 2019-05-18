use serde::{Deserialize, Serialize};
use super::FileDetails;

#[derive(Serialize, Deserialize)]
pub struct FileList {
    pub files: Vec<FileDetails>,
    pub file_updates: Vec<FileUpdate>,
}

#[derive(Serialize, Deserialize)]
pub struct FileUpdate {
    pub old_file_id: u64,
    pub new_file_id: u64,
    pub old_file_name: String,
    pub new_file_name: String,
    pub uploaded_timestamp: u64,
    pub uploaded_time: String,
}
