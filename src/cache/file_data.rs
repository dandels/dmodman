use super::LocalFile;
use crate::api::FileDetails;

use std::cmp::{Ord, Ordering, PartialOrd};
use std::hash::{Hash, Hasher};

use tokio::sync::RwLock;

// Precomputed pairings of LocalFiles with the FileDetails found in FileLists
#[derive(Debug)]
pub struct FileData {
    pub file_id: u64,
    pub local_file: RwLock<LocalFile>,
    pub file_details: FileDetails,
}

impl FileData {
    pub fn new(lf: LocalFile, file_details: FileDetails) -> Self {
        Self {
            file_id: lf.file_id,
            local_file: RwLock::new(lf),
            file_details,
        }
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
        self.file_details.uploaded_timestamp.cmp(&other.file_details.uploaded_timestamp)
    }
}

impl PartialOrd for FileData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
