use super::LocalFile;
use crate::api::{FileDetails, Md5Results};

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

// Precomputed pairings of LocalFiles with the FileDetails found in FileLists
#[derive(Debug)]
pub struct FileData {
    pub file_id: u64,
    pub game: String,
    pub mod_id: u32,
    pub file_name: String,
    pub local_file: LocalFile,
    pub file_details: Option<FileDetails>,
    pub md5results: Option<Md5Results>,
}

impl FileData {
    pub fn new(local_file: LocalFile, file_details: Option<FileDetails>, md5results: Option<Md5Results>) -> Self {
        Self {
            file_id: local_file.file_id,
            game: local_file.game.clone(),
            mod_id: local_file.mod_id,
            file_name: local_file.file_name.clone(),
            local_file,
            file_details,
            md5results,
        }
    }

    pub fn uploaded_timestamp(&self) -> Option<u64> {
        if let Some(fd) = &self.file_details {
            return Some(fd.uploaded_timestamp);
        } else if let Some(res) = &self.md5results {
            return Some(res.file_details.uploaded_timestamp);
        }
        None
    }
}

impl Eq for FileData {}
impl PartialEq for FileData {
    fn eq(&self, other: &Self) -> bool {
        self.file_id == other.file_id
    }
}
impl Hash for FileData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.file_id.hash(state);
    }
}

impl Ord for FileData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.uploaded_timestamp().cmp(&other.uploaded_timestamp())
    }
}

impl PartialOrd for FileData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
