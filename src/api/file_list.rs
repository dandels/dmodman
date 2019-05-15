use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Serialize, Deserialize)]
pub struct FileList {
    pub files: Vec<FileInfo>,
    pub file_updates: Vec<Update>,
}

#[derive(Eq, Serialize, Deserialize)]
pub struct FileInfo {
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
    pub external_virus_scan_url: Option<String>,
    pub description: String,
    pub size_kb: u64,
    pub changelog_html: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Update {
    pub old_file_id: u64,
    pub new_file_id: u64,
    pub old_file_name: String,
    pub new_file_name: String,
    pub uploaded_timestamp: u64,
    pub uploaded_time: String,
}

impl Ord for FileInfo {
    fn cmp(&self, other: &FileInfo) -> Ordering {
        // main, update, optional, old_version or miscellaneous
        if self.category_name == Some("MAIN".to_string()) {
            return Ordering::Less;
        }
        if other.category_name == Some("MAIN".to_string()) {
            return Ordering::Greater;
        }
        if self.category_name == Some("UPDATE".to_string()) {
            return Ordering::Less;
        }
        if other.category_name == Some("UPDATE".to_string()) {
            return Ordering::Greater;
        }
        if self.category_name == Some("OPTIONAL".to_string()) {
            return Ordering::Less;
        }
        if other.category_name == Some("OPTIONAL".to_string()) {
            return Ordering::Greater;
        }
        if self.category_name == Some("OLD_VERSION".to_string()) {
            return Ordering::Less;
        }
        if other.category_name == Some("OLD_VERSION".to_string()) {
            return Ordering::Greater;
        }
        // This case doesn't exist according to the API documentation
        return self.category_name.cmp(&other.category_name);
    }
}

impl PartialOrd for FileInfo {
    fn partial_cmp(&self, other: &FileInfo) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl PartialEq for FileInfo {
    fn eq(&self, other: &FileInfo) -> bool {
        self.category_name == other.category_name
    }
}
