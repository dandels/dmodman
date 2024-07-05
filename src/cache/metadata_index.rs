use super::*;
use crate::config::Config;
use crate::install::*;
use crate::Logger;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type Map<K, V> = Arc<RwLock<HashMap<K, V>>>;

#[derive(Clone)]
pub struct MetadataIndex {
    #[allow(dead_code)]
    config: Arc<Config>,
    #[allow(dead_code)]
    logger: Logger,

    // references to other cache fields
    file_lists: FileLists,
    mod_info_map: ModInfoMap,

    by_file_id: Map<u64, Arc<ModFileMetadata>>,
    by_archive_name: Map<String, Arc<ModFileMetadata>>,
    // (game, mod_id) -> BinaryHeap that keeps the modfiles sorted by timestamp. Used by the update checker.
    pub by_game_and_mod_sorted: Map<String, IndexMap<u32, Vec<Arc<ModFileMetadata>>>>,
}

impl MetadataIndex {
    pub async fn new(config: Arc<Config>, logger: Logger, file_lists: FileLists, mod_info_map: ModInfoMap) -> Self {
        Self {
            config,
            logger: logger.clone(),
            by_file_id: Default::default(),
            by_game_and_mod_sorted: Default::default(),
            by_archive_name: Default::default(),
            file_lists,
            mod_info_map,
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
                    mods_map.insert_sorted(mfdata.mod_id, vec![mfdata.clone()]);
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
    pub async fn fill_mod_file_data(&self, mfd: &ModFileMetadata) {
        {
            if mfd.file_details().await.is_none() {
                if let Some(fd) = self.file_lists.filedetails_for(mfd.game.to_owned(), mfd.mod_id, mfd.file_id).await {
                    mfd.set_file_details(fd).await;
                }
            }
        }
        {
            let mut modinfo_lock = mfd.mod_info.write().await;
            if modinfo_lock.is_none() {
                *modinfo_lock = self.mod_info_map.get(&mfd.game, mfd.mod_id).await;
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
        let mfd = match self.get_by_file_id(&metadata.file_id).await {
            Some(mfd) => mfd,
            None => {
                let mfd = Arc::new(ModFileMetadata::new(
                    metadata.game.clone(),
                    metadata.mod_id,
                    metadata.file_id,
                ));
                if let ArchiveEntry::File(archive) = &archive_entry {
                    mfd.add_archive(archive.clone()).await;
                }
                mfd
            },
        };

        self.fill_mod_file_data(&mfd).await;

        if let ArchiveEntry::File(archive) = &archive_entry {
            mfd.add_archive(archive.clone()).await;
        }

        self.add_to_collections(Some(metadata.file_name.clone()), mfd.clone()).await;
        Some(mfd)
    }

    pub async fn add_installed(&self, dir_name: String, file_id: u64, im: Arc<InstalledMod>) -> Arc<ModFileMetadata> {
        let mfd = self.get_by_file_id(&file_id).await.unwrap_or(Arc::new((&*im).into()));
        mfd.add_installed_dir(dir_name.clone(), im.clone()).await;
        self.fill_mod_file_data(&mfd).await;
        self.add_to_collections(Some(im.installation_file.clone()), mfd.clone()).await;
        mfd
    }

    pub async fn get_by_file_id(&self, file_id: &u64) -> Option<Arc<ModFileMetadata>> {
        self.by_file_id.read().await.get(file_id).cloned()
    }

    pub async fn get_modfiles(&self, game: &str, mod_id: &u32) -> Option<Vec<Arc<ModFileMetadata>>> {
        self.by_game_and_mod_sorted.read().await.get(game).and_then(|mods| mods.get(mod_id).cloned())
    }

    pub async fn get_by_archive_name(&self, name: &String) -> Option<Arc<ModFileMetadata>> {
        self.by_archive_name.read().await.get(name).cloned()
    }

    pub async fn remove_if_unreferenced(&self, file_id: &u64) {
        if let Some(mfd) = self.get_by_file_id(file_id).await {
            if mfd.is_unreferenced().await {
                /* the entry in self.by_archive_name should be taken care of by the archives struct because
                 * mfd.mod_archives is empty. */
                self.by_file_id.write().await.remove(&mfd.file_id);
                let mut games_lock = self.by_game_and_mod_sorted.write().await;
                let mods_in_game = games_lock.get_mut(&mfd.game).unwrap();
                let files = mods_in_game.get_mut(&mfd.mod_id).unwrap();
                let index = files
                    .binary_search_by(|f| f.file_id.cmp(&mfd.file_id))
                    .expect("Should have found file by id {file_id} to delete in game->mods->files map.");
                files.remove(index);
            }
        }
    }
}
