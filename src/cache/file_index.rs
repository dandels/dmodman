use super::{CacheError, Cacheable, FileData, FileLists, LocalFile};
use crate::config::Config;

use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use indexmap::IndexMap;
use tokio::fs;
use tokio::sync::RwLock;

// TODO handle foreign (without associated LocalData) files somehow?
#[derive(Clone)]
pub struct FileIndex {
    /* game: String,
     * mod_id: u32,
     * file_id: u64 */
    pub files: Arc<RwLock<IndexMap<u64, Arc<FileData>>>>,
    /* Maps (game, mod_id) to files in that mod. Use a binary heap that keeps the mods sorted by timestamp.
     * (This avoids sorting the files every time we check for updates.) */
    #[allow(clippy::type_complexity)] // maybe clippy has a point, though
    pub mod_file_mapping: Arc<RwLock<HashMap<(String, u32), BinaryHeap<Arc<FileData>>>>>,
    pub has_changed: Arc<AtomicBool>,
    // TODO read file lists from disk into memory only for games where it's needed
    file_lists: FileLists,
}

impl FileIndex {
    pub async fn new(config: &Config, file_lists: FileLists) -> Result<Self, CacheError> {
        // It's unexpected but possible that FileDetails is missing
        let mut file_index: IndexMap<u64, Arc<FileData>> = IndexMap::new();
        let mut mod_files: HashMap<(String, u32), BinaryHeap<Arc<FileData>>> = HashMap::new();

        /* 1. Iterates through all <mod_file>.json files in the download directory for the current game, skipping those
         *    where the corresponding <mod_file> is missing.
         * 2. Serialize the json files into LocalFile's.
         * 3. Use the file id to map each LocalFile to a FileDetails, stored in the FileData struct.
         * 4. Store the FileData's in a timestamp-sorted binary heap because the update algorithm depends on it. */
        if let Ok(mut file_stream) = fs::read_dir(config.download_dir()).await {
            while let Some(f) = file_stream.next_entry().await? {
                if f.path().is_file() && f.path().extension().and_then(OsStr::to_str) != Some("json") {
                    let json_file = f.path().with_file_name(format!("{}.json", f.file_name().to_string_lossy()));
                    if let Ok(lf) = LocalFile::load(json_file).await {
                        if let Some(file_list) = file_lists.get((&lf.game, lf.mod_id)).await {
                            let file_details = file_list.files.iter().find(|fd| fd.file_id == lf.file_id).unwrap();
                            let file_data = Arc::new(FileData::new(lf.clone(), file_details.clone()));
                            file_index.insert(lf.file_id, file_data.clone());
                            match mod_files.get_mut(&(lf.game.to_string(), lf.mod_id)) {
                                Some(heap) => {
                                    heap.push(file_data);
                                }
                                None => {
                                    let mut heap = BinaryHeap::new();
                                    heap.push(file_data);
                                    mod_files.insert((lf.game.to_string(), lf.mod_id), heap);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            files: Arc::new(RwLock::new(file_index)),
            mod_file_mapping: Arc::new(RwLock::new(mod_files)),
            has_changed: Arc::new(AtomicBool::new(false)),
            file_lists,
        })
    }

    pub async fn add(&self, lf: LocalFile) {
        // TODO sort out this whole missing FileDetails thing
        let file_details = self.file_lists.filedetails_for(&lf).await.unwrap();
        let fdata: Arc<FileData> = FileData::new(lf.clone(), file_details).into();
        self.files.write().await.insert(lf.file_id, fdata.clone());
        if let Some(heap) = self.mod_file_mapping.write().await.get_mut(&(lf.game, lf.mod_id)) {
            heap.push(fdata);
        }
        self.has_changed.store(true, Ordering::Relaxed);
    }
}
