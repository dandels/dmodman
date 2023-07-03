use super::{FileDetails, Queriable};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileList {
    pub files: Vec<FileDetails>,
    pub file_updates: BinaryHeap<FileUpdate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl Eq for FileUpdate {}

impl PartialEq for FileUpdate {
    fn eq(&self, other: &Self) -> bool {
        self.uploaded_timestamp == other.uploaded_timestamp && self.new_file_id == other.new_file_id
    }
}

impl Ord for FileUpdate {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.uploaded_timestamp == other.uploaded_timestamp {
            return self.new_file_id.cmp(&other.new_file_id);
        }
        self.uploaded_timestamp.cmp(&other.uploaded_timestamp)
    }
}

impl PartialOrd for FileUpdate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
