use crate::cache::{Cacheable, MetadataIndex};
use crate::config::DataPath;
use crate::install::installed_mod::*;
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
    pub async fn new(config: Arc<Config>, logger: Logger, metadata_index: MetadataIndex, archives_has_changed: Arc<AtomicBool>) -> Self {
        let mut installed: Vec<(String, ModDirectory)> = vec![];
        let mut by_file_id: Vec<(u64, Arc<InstalledMod>)> = vec![];
        if let Ok(mut install_dir) = fs::read_dir(config.install_dir()).await {
            while let Ok(Some(mod_dir)) = install_dir.next_entry().await {
                let dir_name = mod_dir.file_name().to_string_lossy().to_string();
                if let Ok(mod_dir) = ModDirectory::load(DataPath::ModDirMetadata(&config, &dir_name)).await {
                    match mod_dir {
                        ModDirectory::Nexus(im) => {
                            metadata_index.add_installed(dir_name.clone(), im.file_id, im.clone()).await;
                            installed.push((dir_name, ModDirectory::Nexus(im.clone())));
                            by_file_id.push((im.file_id, im));
                        }
                        _ => {
                            installed.push((dir_name, mod_dir));
                        }
                    }
                }
            }
        }
        Self {
            config,
            logger,
            metadata_index,
            mods: Arc::new(IndexMap::from_iter(installed).into()),
            has_changed: Arc::new(true.into()),
            archives_has_changed,
        }
    }

    pub async fn get(&self, name: &String) -> Option<(String, ModDirectory)> {
        self.mods.read().await.get_key_value(name).map(|(k, v)| (k.clone(), v.clone()))
    }

    pub async fn add(&self, dir_name: String, md: ModDirectory) {
        if let ModDirectory::Nexus(im) = &md {
            self.metadata_index.add_installed(dir_name.clone().clone(), im.file_id, im.clone()).await;
        }
        self.mods.write().await.insert(dir_name, md);
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn delete(&self, dir_name: &String) {
        let mut mods_lock = self.mods.write().await;
        let path = self.config.install_dir().join(dir_name);
        if let Err(e) = fs::remove_dir_all(path).await {
            self.logger.log(format!("Error {e} when removing {dir_name}"));
            return;
        }
        if let Some(mod_dir) = mods_lock.swap_remove(dir_name) {
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
    }
}
