use super::FileDetails;
use serde::{Deserialize, Serialize};

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

impl FileList {
    fn get_by_id(&self, file_id: &u64) -> Option<FileDetails> {
        let mut ret: Vec<FileDetails> = self
            .files
            .clone()
            .into_iter()
            .filter(|x| x.file_id == *file_id)
            .collect();
        return ret.pop();
    }
}

impl Clone for FileUpdate {
    fn clone(&self) -> Self {
        return FileUpdate {
            old_file_id: self.old_file_id,
            new_file_id: self.new_file_id,
            new_file_name: self.new_file_name.clone(),
            old_file_name: self.old_file_name.clone(),
            uploaded_timestamp: self.uploaded_timestamp,
            uploaded_time: self.uploaded_time.clone(),
        };
    }
}
