use crate::cache::{Cacheable, MetadataIndex};
use crate::config::DataPath;
use crate::extract::installed_mod::*;
use crate::{Config, Logger};
use indexmap::IndexMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Installed {
    config: Arc<Config>,
    logger: Logger,
    metadata_index: MetadataIndex,
    pub mods: Arc<RwLock<IndexMap<String, ModDirectory>>>, // Key = Directory name
    pub has_changed: Arc<AtomicBool>,
    archives_has_changed: Arc<AtomicBool>,
}

impl Installed {
    pub async fn new(
        config: Arc<Config>,
        logger: Logger,
        metadata_index: MetadataIndex,
        archives_has_changed: Arc<AtomicBool>,
    ) -> Self {
        let mut installed: IndexMap<String, ModDirectory> = IndexMap::new();
        let install_dir_read = fs::read_dir(config.install_dir()).await;
        if let Ok(load_order) = config.read_load_order() {
            if install_dir_read.is_err() && load_order.is_empty() {
                logger.log("Error: load order is present but installed mods dir does not exist.");
            } else {
                for dir in load_order {
                    match fs::read_dir(config.install_dir().join(&dir)).await {
                        Ok(_) => {
                            add_dir(&config, &metadata_index, dir, &mut installed).await;
                        }
                        Err(_) => {
                            logger.log(format!("Warn: \"{dir}\" is missing but exists in load order."));
                        }
                    }
                }
            }
        }
        if let Ok(mut install_dir) = install_dir_read {
            while let Ok(Some(mod_dir)) = install_dir.next_entry().await {
                let dir_name = mod_dir.file_name().to_string_lossy().to_string();
                if installed.get(&dir_name).is_none() {
                    add_dir(&config, &metadata_index, dir_name, &mut installed).await;
                }
            }
        }
        let ret = Self {
            config,
            logger,
            metadata_index,
            mods: Arc::new(IndexMap::from_iter(installed).into()),
            has_changed: Arc::new(true.into()),
            archives_has_changed,
        };
        ret.save_load_order().await;
        ret
    }

    pub async fn get(&self, name: &str) -> Option<(String, ModDirectory)> {
        self.mods.read().await.get_key_value(name).map(|(k, v)| (k.clone(), v.clone()))
    }

    pub async fn add(&self, dir_name: String, md: ModDirectory) {
        if let ModDirectory::Nexus(im) = &md {
            self.metadata_index.add_installed(dir_name.clone().clone(), im.file_id, im.clone()).await;
        }
        self.mods.write().await.insert(dir_name, md);
        self.has_changed.store(true, Ordering::Relaxed);
        self.save_load_order().await;
    }

    pub async fn delete(&self, dir_name: &String) {
        let mut mods_lock = self.mods.write().await;
        let path = self.config.install_dir().join(dir_name);
        if let Err(e) = fs::remove_dir_all(path).await {
            self.logger.log(format!("Error {e} when removing {dir_name}"));
            return;
        }
        if let Some(mod_dir) = mods_lock.shift_remove(dir_name) {
            if let ModDirectory::Nexus(im) = mod_dir {
                let mfd = self
                    .metadata_index
                    .get_by_file_id(&im.file_id)
                    .await
                    .unwrap_or_else(|| panic!("{} should have been present in the metadata index.", &im.file_id));
                if mfd.remove_installed(dir_name).await {
                    self.archives_has_changed.store(true, Ordering::Relaxed);
                }
                self.metadata_index.remove_if_unreferenced(&mfd.file_id).await;
            }
            self.has_changed.store(true, Ordering::Relaxed);
        } else {
            self.logger
                .log(format!("{dir_name} no longer exists. Please file a bug report if you did not just remove it."));
        }
        self.save_load_order().await;
    }

    async fn save_load_order(&self) {
        if let Err(e) = self.config.save_load_order(self.mods.read().await.keys().cloned().collect()) {
            self.logger.log(format!("Error: unable to save load order: {e}"));
        }
    }

    pub async fn move_to_index(&self, src_index: usize, mut dest_index: usize) {
        {
            let mut lock = self.mods.write().await;
            if lock.is_empty() || src_index >= lock.len() {
                return;
            }
            if dest_index >= lock.len() {
                dest_index = 0;
            }
            //dest_index = std::cmp::min(lock.len().saturating_sub(1), dest_index);
            if src_index.abs_diff(dest_index) == 1 {
                lock.swap_indices(src_index, dest_index);
            } else {
                lock.move_index(src_index, dest_index);
            }
        }
        // TODO save load order without causing too many writes
        // maybe start a task with a delay that gets restarted every time user reorders mods
        self.has_changed.store(true, Ordering::Relaxed);
        self.save_load_order().await;
    }
}

async fn add_dir(
    config: &Arc<Config>,
    metadata_index: &MetadataIndex,
    dir_name: String,
    installed: &mut IndexMap<String, ModDirectory>,
) {
    if let Ok(mod_dir) = ModDirectory::load(DataPath::ModDirMetadata(config, &dir_name)).await {
        match mod_dir {
            ModDirectory::Nexus(im) => {
                metadata_index.add_installed(dir_name.clone(), im.file_id, im.clone()).await;
                installed.insert(dir_name, ModDirectory::Nexus(im.clone()));
            }
            _ => {
                installed.insert(dir_name, mod_dir);
            }
        }
    }
}
