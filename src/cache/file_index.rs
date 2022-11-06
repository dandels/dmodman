use super::{CacheError, Cacheable, FileData, FileListCache, LocalFile};
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

// TODO handle foreign (without associated LocalData) files somehow
#[derive(Clone)]
pub struct Files {
    // This is the list of downloaded mods.
    pub file_index: Arc<RwLock<IndexMap<u64, Arc<FileData>>>>,
    /* Maps (game, mod_id) to files in that mod.
     * It uses a binary heap that keeps the mods sorted by timestamp */
    pub mod_files: Arc<RwLock<HashMap<(String, u32), BinaryHeap<Arc<FileData>>>>>,
    pub has_changed: Arc<AtomicBool>,
    // TODO read file lists from disk into memory only for games where it's needed
    file_lists: FileListCache,
}

impl Files {
    /* This iterates through all files in the download directory for the current game. For each file, if the
     * corresponding json file exists, it deserializes the json file into a LocalFile.
     * It then checks in the FileList cache if there exists a corresponding FileDetails for that file.
     * The result is a pair of LocalFile, FileDetails, stored in the FileData struct.
     * Also creates a mapping of mod_id -> FileDatas, so that iterating through files in a mod doesn't require multiple
     * table lookups. */
    pub async fn new(config: &Config, file_lists: FileListCache) -> Result<Self, CacheError> {
        // It's unexpected but possible that FileDetails is missing
        let mut file_index: IndexMap<u64, Arc<FileData>> = IndexMap::new();
        let mut mod_files: HashMap<(String, u32), BinaryHeap<Arc<FileData>>> = HashMap::new();

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
            file_index: Arc::new(RwLock::new(file_index)),
            mod_files: Arc::new(RwLock::new(mod_files)),
            has_changed: Arc::new(AtomicBool::new(false)),
            file_lists,
        })
    }

    pub async fn add(&self, lf: LocalFile) {
        // TODO sort out this whole missing FileDetails thing
        let file_details = self.file_lists.filedetails_for(&lf).await.unwrap();
        let fdata: Arc<FileData> = FileData::new(lf.clone(), file_details).into();
        self.file_index.write().await.insert(lf.file_id, fdata.clone());
        if let Some(heap) = self.mod_files.write().await.get_mut(&(lf.game, lf.mod_id)) {
            heap.push(fdata);
        }
        self.has_changed.store(true, Ordering::Relaxed);
    }
}
