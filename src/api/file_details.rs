use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Eq, Serialize, Deserialize)]
pub struct FileDetails {
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

impl Ord for FileDetails {
    fn cmp(&self, other: &FileDetails) -> Ordering {
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
        if self.category_name == Some("MISCELLANEOUS".to_string()) {
            return Ordering::Less;
        }
        if other.category_name == Some("MISCELLANEOUS".to_string()) {
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

impl PartialOrd for FileDetails {
    fn partial_cmp(&self, other: &FileDetails) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl PartialEq for FileDetails {
    fn eq(&self, other: &FileDetails) -> bool {
        self.category_name == other.category_name
    }
}

impl Clone for FileDetails {
    fn clone(&self) -> Self {
        FileDetails {
            file_id: self.file_id,
            name: self.name.clone(),
            version: self.version.clone(),
            category_id: self.category_id,
            category_name: self.category_name.clone(),
            is_primary: self.is_primary,
            size: self.size,
            file_name: self.file_name.clone(),
            uploaded_timestamp: self.uploaded_timestamp,
            uploaded_time: self.uploaded_time.clone(),
            mod_version: self.mod_version.clone(),
            external_virus_scan_url: self.external_virus_scan_url.clone(),
            description: self.description.clone(),
            size_kb: self.size_kb,
            changelog_html: self.changelog_html.clone(),
        }
    }
}
