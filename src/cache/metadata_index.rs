use super::*;
use crate::config::Config;
use crate::install::*;
use crate::Logger;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// efficient ways to look up data pointing to the same Arc<ModFileData>'s

type Map<K, V> = Arc<RwLock<HashMap<K, V>>>;

#[derive(Clone)]
pub struct MetadataIndex {
    config: Config,
    logger: Logger,

    // references to other cache fields
    file_lists: FileLists,
    md5result_map: Md5ResultMap,

    pub by_file_id: Map<u64, Arc<ModFileMetadata>>,
    pub by_archive_name: Map<String, Arc<ModFileMetadata>>,
    // (game, mod_id) -> BinaryHeap that keeps the modfiles sorted by timestamp. Used by the update checker.
    pub by_game_and_mod_sorted: Map<String, IndexMap<u32, Vec<Arc<ModFileMetadata>>>>,
    pub has_changed: Arc<AtomicBool>, // should the list be re-rendered
}

impl MetadataIndex {
    pub async fn new(config: Config, logger: Logger, file_lists: FileLists, md5result_map: Md5ResultMap) -> Self {
        Self {
            config,
            logger: logger.clone(),
            by_file_id: Default::default(),
            by_game_and_mod_sorted: Default::default(),
            by_archive_name: Default::default(),
            has_changed: Arc::new(AtomicBool::new(true)),
            file_lists,
            md5result_map,
        }
    }

    async fn add_to_collections(&self, archive_name: Option<String>, mfdata: Arc<ModFileMetadata>) {
        self.by_file_id.write().await.insert(mfdata.file_id, mfdata.clone());
        if let Some(arch_name) = archive_name {
            self.by_archive_name.write().await.insert(arch_name, mfdata.clone());
        }

        let mut game_map_lock = self.by_game_and_mod_sorted.write().await;
        match game_map_lock.get_mut(&mfdata.game) {
            Some(mods_map) => match mods_map.get_mut(&mfdata.mod_id) {
                Some(files) => {
                    let insertion_index = files.partition_point(|f| mfdata.file_id < f.file_id);
                    files.insert(insertion_index, mfdata.clone());
                }
                None => {
                    mods_map.insert(mfdata.mod_id, vec![mfdata.clone()]);
                }
            },
            None => {
                let mut mods_map = IndexMap::new();
                mods_map.insert(mfdata.mod_id, vec![mfdata.clone()]);
                game_map_lock.insert(mfdata.game.clone(), mods_map);
            }
        }
    }

    // needs a better name, we lack (circular) references to other cache structs to fill all fields
    pub async fn fill_mod_file_data(&self, game: &String, mod_id: u32, file_id: u64, mfdata: &ModFileMetadata) {
        {
            let mut fd_lock = mfdata.file_details.write().await;
            if fd_lock.is_none() {
                *fd_lock = self.file_lists.filedetails_for(game.clone(), mod_id, file_id).await;
            }
        }
        {
            let mut md5_lock = mfdata.md5results.write().await;
            if md5_lock.is_none() {
                *md5_lock = self.md5result_map.get(game.clone(), file_id).await;
            }
        }
    }

    pub async fn try_add_mod_archive(&self, archive: Arc<ArchiveFile>) -> Option<Arc<ModFileMetadata>> {
        if let None = archive.mod_data {
            return None;
        }
        let metadata = archive.mod_data.as_ref().unwrap();
        let ArchiveMetadata {
            ref file_name,
            ref game,
            mod_id,
            file_id,
            ..
        } = metadata.as_ref();
        let game = game.clone();
        let file_name = file_name.clone();

        let mfdata = match self.get_by_file_id(&metadata.file_id).await {
            Some(mfdata) => mfdata,
            None => Arc::new(ModFileMetadata::new(
                metadata.game.clone(),
                metadata.mod_id,
                metadata.file_id,
                None,
                None,
                None,
                Some(archive.clone()),
                InstallStatus::Downloaded,
            )),
        };
        self.fill_mod_file_data(&game, *mod_id, *file_id, &mfdata).await;
        {
            let mut lock = mfdata.mod_archives.write().await;
            lock.insert(archive.file_name.clone(), archive.clone());
        }

        self.add_to_collections(Some(file_name), mfdata.clone()).await;
        self.has_changed.store(true, Ordering::Relaxed);
        Some(mfdata)
    }

    pub async fn add_installed(&self, dir_name: String, file_id: u64, im: Arc<InstalledMod>) -> Arc<ModFileMetadata> {
        let mfd = match self.get_by_file_id(&file_id).await {
            Some(mfd) => mfd,
            None => ModFileMetadata::new(
                im.game.clone(),
                im.mod_id,
                im.file_id,
                None,
                Some((dir_name.clone(), im.clone())),
                None,
                None,
                InstallStatus::Installed,
            )
            .into(),
        };
        self.fill_mod_file_data(&mfd.game, mfd.mod_id, mfd.file_id, &mfd).await;
        mfd.installed_mods.write().await.insert(dir_name, im.clone());
        self.add_to_collections(Some(im.installation_file.clone()), mfd.clone()).await;
        mfd
    }

    pub async fn get_by_file_id(&self, file_id: &u64) -> Option<Arc<ModFileMetadata>> {
        self.by_file_id.read().await.get(file_id).cloned()
    }

    pub async fn get_modfiles(&self, game: &String, mod_id: &u32) -> Option<Vec<Arc<ModFileMetadata>>> {
        self.by_game_and_mod_sorted.read().await.get(game).and_then(|mods| mods.get(mod_id).cloned())
    }

    pub async fn get_by_archive_name(&self, name: &String) -> Option<Arc<ModFileMetadata>> {
        self.by_archive_name.read().await.get(name).cloned()
    }

    // TODO disabled because of race condition with inotify
    //// Return game, mod id, files for the index selected in the UI.
    //pub async fn get_game_mod_files_by_index(&self, index: usize) -> (String, u32, BinaryHeap<Arc<ModFileData>>) {
    //    // If the unwraps here fail then there is a bug elsewhere causing inconsistent cache state
    //    let lock = self.files_sorted.read().await;
    //    let lf = &lock.get(index).unwrap().mod_archive;
    //    let files = self.game_to_mods_map.read().await.get(&lf.game).and_then(|mods| mods.get(&lf.mod_id).cloned());
    //    (lf.game.clone(), lf.mod_id, files.unwrap())
    //}

    // Delete a file and its metadata based on its index in files_sorted.
    //pub async fn delete_by_index(&self, i: usize) -> Result<(), std::io::Error> {
    //    let mut fs_lock = self.files_sorted.write().await;
    //    let mut game_mods_lock = self.game_to_mods_map.write().await;
    //    let mut files_lock = self.file_id_map.write().await;
    //    let fd = fs_lock.get(&i).unwrap().clone();
    //    let id_to_delete = fs_lock.get(&i).unwrap().file_id;

    //    files_lock.remove(&id_to_delete);

    //    fs_lock.swap_remove(&i);

    //    let mods_map = game_mods_lock.get_mut(&fd.mod_archive.game).unwrap();
    //    let heap = mods_map.get_mut(&fd.mod_archive.mod_id).unwrap();
    //    heap.retain(|fdata| fdata.file_id != id_to_delete);
    //    if heap.is_empty() {
    //        mods_map.shift_remove(&fd.mod_archive.mod_id);
    //    }
    //    if mods_map.is_empty() {
    //        game_mods_lock.remove(&fd.mod_archive.game);
    //    }

    //    let mut path = self.config.path_for(DataType::ModArchive(&fd.mod_archive));
    //    fs::remove_file(&path).await?;
    //    path.pop();
    //    path.push(&fd.mod_archive.file_name);
    //    fs::remove_file(path).await?;
    //    self.has_changed.store(true, Ordering::Relaxed);
    //    Ok(())
    //}
}
