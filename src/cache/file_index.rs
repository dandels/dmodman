use super::{CacheError, Cacheable, FileData, FileLists, LocalFile};
use crate::config::Config;

use indexmap::IndexMap;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::UNIX_EPOCH;
use tokio::sync::RwLock;

/* This is the struct to query if you want to find out something about a file.
 *
 * - LocalFile: a metadata file with the filename, game, mod_id, file_id, and update status
 * - FileDetails: an API response containing information about a specific file
 * - FileData: a struct that maps a file_id to a LocalFile and its FileDetails.
 */

#[derive(Clone)]
pub struct FileIndex {
    // maps file_id to FileData
    pub file_id_map: Arc<RwLock<HashMap<u64, Arc<FileData>>>>,
    // (game, mod_id) -> BinaryHeap that keeps the modfiles sorted by timestamp. Used by the update checker.
    #[allow(clippy::type_complexity)]
    pub game_to_mods_map: Arc<RwLock<HashMap<String, IndexMap<u32, BinaryHeap<Arc<FileData>>>>>>,
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
        let mut game_mods_map: HashMap<String, IndexMap<u32, BinaryHeap<Arc<FileData>>>> = HashMap::new();
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
                    if let Some(file_list) = file_lists.get(lf.game.clone(), lf.mod_id).await {
                        let file_details = file_list.files.iter().find(|fd| fd.file_id == lf.file_id).unwrap();
                        let file_data = Arc::new(FileData::new(lf.clone(), file_details.clone()));
                        file_index.insert(lf.file_id, file_data.clone());
                        files_sorted.push(file_data.clone());

                        match game_mods_map.get_mut(&lf.game) {
                            Some(mods_map) => {
                                match mods_map.get_mut(&lf.mod_id) {
                                    Some(heap) => heap.push(file_data),
                                    None => {
                                        mods_map.insert(lf.mod_id, BinaryHeap::from([file_data]));
                                    }
                                }
                            }
                            None => {
                                let mut map = IndexMap::new();
                                map.insert(lf.mod_id, BinaryHeap::from([file_data]));
                                game_mods_map.insert(lf.game, map);
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            file_id_map: Arc::new(RwLock::new(file_index)),
            game_to_mods_map: Arc::new(RwLock::new(game_mods_map)),
            files_sorted: Arc::new(RwLock::new(files_sorted)),
            has_changed: Arc::new(AtomicBool::new(true)),
            file_lists,
        })
    }

    pub async fn add(&self, lf: LocalFile) {
        // TODO handle missing FileDetails gracefully
        let file_details = self.file_lists.filedetails_for(&lf).await.unwrap();
        let fdata: Arc<FileData> = FileData::new(lf.clone(), file_details).into();
        self.file_id_map.write().await.insert(lf.file_id, fdata.clone());

        let mut game_map_lock = self.game_to_mods_map.write().await;
        match game_map_lock.get_mut(&lf.game) {
            Some(mods_map) => match mods_map.get_mut(&lf.mod_id) {
                Some(heap) => {
                    heap.push(fdata.clone());
                }
                None => {
                    mods_map.insert(lf.mod_id, BinaryHeap::from([fdata.clone()]));
                }
            },
            None => {
                let mut mods_map = IndexMap::new();
                mods_map.insert(lf.mod_id, BinaryHeap::from([fdata.clone()]));
                game_map_lock.insert(lf.game, mods_map);
            }
        }

        self.files_sorted.write().await.push(fdata);
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn get_by_filename(&self, name: &str) -> Option<Arc<FileData>> {
        let lock = self.files_sorted.read().await;
        for fd in lock.iter() {
            let lf = fd.local_file.read().await;
            if lf.file_name == name {
                return Some(fd.clone());
            }
        }
        None
    }
}
