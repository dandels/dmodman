use super::{CacheError, Cacheable, FileData, FileLists, LocalFile};
use crate::config::Config;

use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::UNIX_EPOCH;

use std::fs;
use tokio::sync::RwLock;

// Contains various data structures to efficiently look up FileData
#[derive(Clone)]
pub struct FileIndex {
    // maps file_id to FileData
    pub file_id_map: Arc<RwLock<HashMap<u64, Arc<FileData>>>>,
    // (game, mod_id) -> BinaryHeap that keeps the modfiles sorted by timestamp. Used by the update checker.
    #[allow(clippy::type_complexity)]
    pub mod_file_map: Arc<RwLock<HashMap<(String, u32), BinaryHeap<Arc<FileData>>>>>,
    // used by the UI
    pub files_sorted: Arc<RwLock<Vec<Arc<FileData>>>>,
    // should the list be re-rendered
    pub has_changed: Arc<AtomicBool>,
    // reference to FileLists (which uses Arc internally)
    file_lists: FileLists,
}

impl FileIndex {
    pub async fn new(config: &Config, file_lists: FileLists) -> Result<Self, CacheError> {
        // It's unexpected but possible that FileDetails is missing
        let mut file_index: HashMap<u64, Arc<FileData>> = HashMap::new();
        let mut mod_files: HashMap<(String, u32), BinaryHeap<Arc<FileData>>> = HashMap::new();
        let mut files_sorted: Vec<Arc<FileData>> = vec![];

        /* 1. Iterates through all <mod_file>.json files in the download directory for the current game, skipping those
         *    where the corresponding <mod_file> is missing.
         * 2. Serialize the json files into LocalFile's.
         * 3. Use the file id to map each LocalFile to a FileDetails, stored in the FileData struct.
         * 4. Store the FileData's in a timestamp-sorted binary heap because the update algorithm depends on it. */

        // Sort files by creation time
        let mut dir_entries: Vec<_> = match fs::read_dir(config.download_dir()) {
            Ok(rd) => rd.map(|f| f.unwrap()).collect(),
            Err(_) => vec![],
        };
        dir_entries.sort_by_key(|f| match f.metadata() {
            Ok(md) => md.created().unwrap(),
            Err(_) => UNIX_EPOCH,
        });

        for f in dir_entries {
            if f.path().is_file() && f.path().extension().and_then(OsStr::to_str) != Some("json") {
                let json_file = f.path().with_file_name(format!("{}.json", f.file_name().to_string_lossy()));
                if let Ok(lf) = LocalFile::load(json_file).await {
                    if let Some(file_list) = file_lists.get((&lf.game, lf.mod_id)).await {
                        let file_details = file_list.files.iter().find(|fd| fd.file_id == lf.file_id).unwrap();
                        let file_data = Arc::new(FileData::new(lf.clone(), file_details.clone()));
                        file_index.insert(lf.file_id, file_data.clone());
                        files_sorted.push(file_data.clone());
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

        Ok(Self {
            file_id_map: Arc::new(RwLock::new(file_index)),
            mod_file_map: Arc::new(RwLock::new(mod_files)),
            files_sorted: Arc::new(RwLock::new(files_sorted)),
            has_changed: Arc::new(AtomicBool::new(false)),
            file_lists,
        })
    }

    pub async fn add(&self, lf: LocalFile) {
        // TODO handle missing FileDetails gracefully
        let file_details = self.file_lists.filedetails_for(&lf).await.unwrap();
        let fdata: Arc<FileData> = FileData::new(lf.clone(), file_details).into();
        self.file_id_map.write().await.insert(lf.file_id, fdata.clone());
        if let Some(heap) = self.mod_file_map.write().await.get_mut(&(lf.game, lf.mod_id)) {
            heap.push(fdata.clone());
        }
        self.files_sorted.write().await.push(fdata);
        self.has_changed.store(true, Ordering::Relaxed);
    }
}
