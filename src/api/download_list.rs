use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DownloadList {
    pub files: Vec<File>,
    pub file_updates: Vec<Update>
}

#[derive(Serialize, Deserialize)]
pub struct File{
      pub file_id: u64,
      pub name: String,
      pub version: String,
      pub category_id: u32,
      pub category_name: Option<String>,
      pub is_primary: bool,
      pub size: u64,
      pub file_name: String,
      pub uploaded_timestamp: u64,
      pub uploaded_time: String,
      pub mod_version: String,
      pub external_virus_scan_url: Option<String>,
      pub description: String,
      pub size_kb: u64,
      pub changelog_html: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct Update {
      pub old_file_id: u64,
      pub new_file_id: u64,
      pub old_file_name: String,
      pub new_file_name: String,
      pub uploaded_timestamp: u64,
      pub uploaded_time: String
}
