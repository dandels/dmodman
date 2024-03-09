use crate::api::Queriable;
use crate::cache::Cacheable;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileList {
    pub files: Vec<Arc<FileDetails>>,
    pub file_updates: Vec<FileUpdate>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileDetails {
    pub id: (u64, u32), // file_id and game_id
    pub file_id: u64,
    pub name: String,
    pub version: Option<String>,
    pub category_id: u32,
    pub category_name: Option<String>,
    pub is_primary: bool,
    pub size: u64,
    pub file_name: String,
    pub uploaded_timestamp: u64,
    pub uploaded_time: String,
    pub mod_version: Option<String>,
    #[serde(skip)]
    pub external_virus_scan_url: Option<String>,
    #[serde(skip)]
    pub description: Option<String>,
    pub size_kb: u64,
    #[serde(skip)]
    pub changelog_html: Option<String>,
}

impl Cacheable for FileList {}
impl Queriable for FileList {
    const FORMAT_STRING: &'static str = "games/{}/mods/{}/files.json";
}

impl Eq for FileDetails {}

impl PartialEq for FileDetails {
    fn eq(&self, other: &Self) -> bool {
        self.file_id == other.file_id
    }
}

impl Ord for FileDetails {
    fn cmp(&self, other: &Self) -> Ordering {
        self.file_id.cmp(&other.file_id)
    }
}

impl PartialOrd for FileDetails {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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
