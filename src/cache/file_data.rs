use super::LocalFile;
use crate::api::FileDetails;

use std::cmp::{Ord, Ordering, PartialOrd};
use std::hash::{Hash, Hasher};

use tokio::sync::RwLock;

// Precomputed pairings of LocalFiles with the FileDetails found in FileLists
pub struct FileData {
    file_id: u64,
    pub local_file: RwLock<LocalFile>,
    pub file_details: Option<FileDetails>,
}

impl FileData {
    pub fn new(lf: LocalFile, file_details: Option<FileDetails>) -> Self {
        Self {
            file_id: lf.file_id,
            local_file: RwLock::new(lf).into(),
            file_details,
        }
    }
}

impl PartialEq for FileData {
    fn eq(&self, other: &Self) -> bool {
        self.file_id == other.file_id
    }
}
impl Eq for FileData {}
impl Hash for FileData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.file_id.hash(state);
    }
}

impl Ord for FileData {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.file_details {
            Some(self_fd) => match other.file_details {
                Some(other_fd) => self_fd.uploaded_timestamp.cmp(&other_fd.uploaded_timestamp),
                None => Ordering::Greater,
            },
            None => match other.file_details {
                Some(other_fd) => Ordering::Less,
                None => self.file_id.cmp(&other.file_id),
            },
        }
    }
}

impl PartialOrd for FileData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
