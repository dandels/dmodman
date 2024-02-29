use std::collections::{BinaryHeap, HashMap};
use std::ffi::OsStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use indexmap::IndexMap;
use tokio::fs;
use tokio::sync::RwLock;

use super::{CacheError, Cacheable, FileData, FileLists, LocalFile, Md5ResultMap};
use crate::config::{Config, DataType};
use crate::Logger;

/* This is the struct to query if you want to find out something about a file.
 *
 * - LocalFile: a metadata file with the filename, game, mod_id, file_id, and update status
 * - FileDetails: an API response containing information about a specific file
 * - FileData: a struct that maps a file_id to a LocalFile and its FileDetails.
 */

type Map<K, V> = Arc<RwLock<HashMap<K, V>>>;

#[derive(Clone)]
pub struct FileIndex {
    config: Config,
    // maps file_id to FileData
    file_id_map: Map<u64, Arc<FileData>>,
    // (game, mod_id) -> BinaryHeap that keeps the modfiles sorted by timestamp. Used by the update checker.
    pub game_to_mods_map: Map<String, IndexMap<u32, BinaryHeap<Arc<FileData>>>>,
    // used by the UI
    pub files_sorted: Arc<RwLock<Vec<Arc<FileData>>>>,
    // should the list be re-rendered
    pub has_changed: Arc<AtomicBool>,
    file_lists: FileLists,
    md5result_map: Md5ResultMap,
}

impl FileIndex {
    pub async fn new(
        config: Config,
        logger: Logger,
        file_lists: FileLists,
        md5result_map: Md5ResultMap,
    ) -> Result<Self, CacheError> {
        // It's unexpected but possible that FileDetails is missing
        let mut file_index: HashMap<u64, Arc<FileData>> = HashMap::new();
        let mut game_mods_map: HashMap<String, IndexMap<u32, BinaryHeap<Arc<FileData>>>> = HashMap::new();
        let mut files_sorted: Vec<Arc<FileData>> = vec![];

        /* 1. Iterates through all <mod_file>.json files in the download directory for the current game, skipping those
         *    where the corresponding <mod_file> is missing.
         * 2. Serialize the json files into LocalFile's.
         * 3. Use the file id to map each LocalFile to a FileDetails, stored in the FileData struct.
         * 4. Store the FileData's in a timestamp-sorted binary heap because the update algorithm depends on it. */

        /* Sort files by creation time.
         * This is easier with std::fs and we always block on Cache initialization anyway. */
        let mut dir_entries: Vec<_> = match std::fs::read_dir(config.download_dir()) {
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
                match LocalFile::load(json_file).await {
                    Ok(lf) => {
                        if let Some(file_list) = file_lists.get(lf.game.clone(), lf.mod_id).await {
                            let file_details = file_list.files.iter().find(|fd| fd.file_id == lf.file_id).unwrap();
                            let md5res = md5result_map.get(lf.game.clone(), lf.file_id).await;
                            let file_data = Arc::new(FileData::new(lf, Some(file_details.clone()), md5res));
                            file_index.insert(file_data.local_file.file_id, file_data.clone());
                            files_sorted.push(file_data.clone());

                            match game_mods_map.get_mut(&file_data.local_file.game) {
                                Some(mods_map) => match mods_map.get_mut(&file_data.local_file.mod_id) {
                                    Some(heap) => heap.push(file_data),
                                    None => {
                                        mods_map.insert(file_data.local_file.mod_id, BinaryHeap::from([file_data]));
                                    }
                                },
                                None => {
                                    let mut map = IndexMap::new();
                                    map.insert(file_data.local_file.mod_id, BinaryHeap::from([file_data.clone()]));
                                    game_mods_map.insert(file_data.local_file.game.clone(), map);
                                }
                            }
                        }
                    }
                    // TODO archive is missing its LocalFile
                    Err(e) => {
                        logger.log(format!("Archive {:?} is missing its metadata:", f.path().file_name().unwrap()));
                        logger.log(format!("    {e}"));
                    }
                }
            }
        }

        Ok(Self {
            config,
            file_id_map: Arc::new(RwLock::new(file_index)),
            game_to_mods_map: Arc::new(RwLock::new(game_mods_map)),
            files_sorted: Arc::new(RwLock::new(files_sorted)),
            has_changed: Arc::new(AtomicBool::new(true)),
            file_lists,
            md5result_map,
        })
    }

    pub async fn add(&self, lf: LocalFile) {
        // TODO handle missing FileDetails gracefully
        let file_details = self.file_lists.filedetails_for(lf.game.clone(), lf.mod_id, lf.file_id).await;
        let md5res = self.md5result_map.get(lf.game.clone(), lf.file_id).await;
        let fdata: Arc<FileData> = FileData::new(lf, file_details, md5res).into();
        self.file_id_map.write().await.insert(fdata.file_id, fdata.clone());

        let mut game_map_lock = self.game_to_mods_map.write().await;
        match game_map_lock.get_mut(&fdata.game) {
            Some(mods_map) => match mods_map.get_mut(&fdata.mod_id) {
                Some(heap) => {
                    heap.push(fdata.clone());
                }
                None => {
                    mods_map.insert(fdata.mod_id, BinaryHeap::from([fdata.clone()]));
                }
            },
            None => {
                let mut mods_map = IndexMap::new();
                mods_map.insert(fdata.mod_id, BinaryHeap::from([fdata.clone()]));
                game_map_lock.insert(fdata.game.clone(), mods_map);
            }
        }

        self.files_sorted.write().await.push(fdata);
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn get_by_file_id(&self, file_id: &u64) -> Option<Arc<FileData>> {
        self.file_id_map.read().await.get(file_id).cloned()
    }

    pub async fn get_by_index(&self, index: usize) -> Arc<FileData> {
        self.files_sorted.read().await.get(index).unwrap().clone()
    }

    pub async fn get_modfiles(&self, game: &String, mod_id: &u32) -> Option<BinaryHeap<Arc<FileData>>> {
        self.game_to_mods_map.read().await.get(game).and_then(|mods| mods.get(mod_id).cloned())
    }

    // Return game, mod id, files for the index selected in the UI.
    pub async fn get_game_mod_files_by_index(&self, index: usize) -> (String, u32, BinaryHeap<Arc<FileData>>) {
        // If the unwraps here fail then there is a bug elsewhere causing inconsistent cache state
        let lock = self.files_sorted.read().await;
        let lf = &lock.get(index).unwrap().local_file;
        let files = self.game_to_mods_map.read().await.get(&lf.game).and_then(|mods| mods.get(&lf.mod_id).cloned());
        (lf.game.clone(), lf.mod_id, files.unwrap())
    }

    pub async fn get_by_filename(&self, name: &str) -> Option<Arc<FileData>> {
        let lock = self.files_sorted.read().await;
        for fd in lock.iter() {
            if fd.local_file.file_name == name {
                return Some(fd.clone());
            }
        }
        None
    }

    // Delete a file and its metadata based on its index in files_sorted.
    pub async fn delete_by_index(&self, i: usize) -> Result<(), std::io::Error> {
        let mut fs_lock = self.files_sorted.write().await;
        let mut game_mods_lock = self.game_to_mods_map.write().await;
        let mut files_lock = self.file_id_map.write().await;
        let fd = fs_lock.get(i).unwrap().clone();
        let id_to_delete = fs_lock.get(i).unwrap().file_id;

        files_lock.remove(&id_to_delete);

        fs_lock.remove(i);

        let mods_map = game_mods_lock.get_mut(&fd.local_file.game).unwrap();
        let heap = mods_map.get_mut(&fd.local_file.mod_id).unwrap();
        heap.retain(|fdata| fdata.file_id != id_to_delete);
        if heap.is_empty() {
            mods_map.shift_remove(&fd.local_file.mod_id);
        }
        if mods_map.is_empty() {
            game_mods_lock.remove(&fd.local_file.game);
        }

        let mut path = self.config.path_for(DataType::LocalFile(&fd.local_file));
        fs::remove_file(&path).await?;
        path.pop();
        path.push(&fd.local_file.file_name);
        fs::remove_file(path).await?;
        self.has_changed.store(true, Ordering::Relaxed);
        Ok(())
    }
}
