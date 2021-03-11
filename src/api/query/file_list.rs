use super::{FileDetails, Queriable};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileList {
    pub files: Vec<FileDetails>,
    pub file_updates: Vec<FileUpdate>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileUpdate {
    pub old_file_id: u64,
    pub new_file_id: u64,
    pub old_file_name: String,
    pub new_file_name: String,
    pub uploaded_timestamp: u64,
    pub uploaded_time: String,
}

#[async_trait]
impl Queriable for FileList {
    const FORMAT_STRING: &'static str = "games/{}/mods/{}/files.json";
}
