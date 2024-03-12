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
    #[allow(dead_code)]
    config: Config,
    #[allow(dead_code)]
    logger: Logger,

    // references to other cache fields
    file_lists: FileLists,
    md5result_map: Md5ResultMap,

    pub by_file_id: Map<u64, Arc<ModFileMetadata>>, // is this one needed?
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

    pub async fn add_to_collections(&self, archive_name: Option<String>, mfdata: Arc<ModFileMetadata>) {
        self.by_file_id.write().await.insert(mfdata.file_id, mfdata.clone());
        if let Some(archive_name) = archive_name {
            self.by_archive_name.write().await.insert(archive_name, mfdata.clone());
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

    // TODO (circular) references to other cache structs to fill all fields within this one function
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

    pub async fn try_add_mod_archive(&self, archive_entry: ArchiveEntry) -> Option<Arc<ModFileMetadata>> {
        let metadata = match &archive_entry {
            ArchiveEntry::File(archive) => {
                match &archive.mod_data {
                    Some(metadata) => metadata,
                    None => return None, // return early if the metadata is missing
                }
            }
            ArchiveEntry::MetadataOnly(metadata) => metadata,
        };
        let ArchiveMetadata {
            ref file_name,
            ref game,
            mod_id,
            file_id,
            .. // avoid touching UpdateStatusWrapper
        } = metadata.as_ref();
        let game = game.clone();
        let file_name = file_name.clone();

        let mfdata = match self.get_by_file_id(&metadata.file_id).await {
            Some(mfdata) => mfdata,
            None => match &archive_entry {
                ArchiveEntry::File(archive) => {
                    Arc::new(ModFileMetadata::new(
                        metadata.game.clone(),
                        metadata.mod_id,
                        metadata.file_id,
                        None,
                        None,
                        None,
                        Some(archive.clone()),
                    ))
                }
                ArchiveEntry::MetadataOnly(_) => {
                    Arc::new(ModFileMetadata::new(
                        metadata.game.clone(),
                        metadata.mod_id,
                        metadata.file_id,
                        None,
                        None,
                        None,
                        None,
                    ))
                }
            }
        };

        self.fill_mod_file_data(&game, *mod_id, *file_id, &mfdata).await;

        if let ArchiveEntry::File(archive) = &archive_entry {
            let mut lock = mfdata.mod_archives.write().await;
            lock.insert(metadata.file_name.clone(), archive.clone());
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

    pub async fn delete_if_unreferenced(&self, file_id: &u64) {
        let mut has_reference = false;
        if let Some(mfd) = self.get_by_file_id(file_id).await {
            if !mfd.installed_mods.read().await.is_empty() {
                has_reference = true;
            } else if !mfd.mod_archives.read().await.is_empty() {
                has_reference = true;
            }
            if !has_reference {
                /* the entry in self.by_archive_name should be taken care of by the archives struct because
                 * md.mod_archives is empty. */
                self.by_file_id.write().await.remove(&mfd.file_id);
                let mut games_lock = self.by_game_and_mod_sorted.write().await;
                let mods_in_game = games_lock.get_mut(&mfd.game).unwrap();
                let files = mods_in_game.get_mut(&mfd.mod_id).unwrap();
                let index = files.binary_search_by(|f| f.file_id.cmp(&mfd.file_id)).
                       expect("Should have found file by id {file_id} to delete in game->mods->files map.");
                files.remove(index);
            }
        }
    }
}
