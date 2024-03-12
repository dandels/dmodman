use crate::cache::{Cacheable, MetadataIndex};
use crate::install::installed_mod::*;
use crate::{Config, Logger};
use indexmap::IndexMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Installed {
    config: Config,
    logger: Logger,
    metadata_index: MetadataIndex,
    pub mods: Arc<RwLock<IndexMap<String, Arc<ModDirectory>>>>, // Key = Directory name
    pub has_changed: Arc<AtomicBool>,
}

impl Installed {
    pub async fn new(config: Config, logger: Logger, metadata_index: MetadataIndex) -> Self {
        let mut installed: Vec<(String, Arc<ModDirectory>)> = vec![];
        let mut by_file_id: Vec<(u64, Arc<InstalledMod>)> = vec![];
        if let Ok(mut install_dir) = fs::read_dir(config.install_dir()).await {
            while let Ok(Some(mod_dir)) = install_dir.next_entry().await {
                let dir_name = mod_dir.file_name().to_string_lossy().to_string();
                match InstalledMod::load(mod_dir.path().join(".dmodman-meta.json")).await {
                    Ok(im) => {
                        let im = Arc::new(im);
                        metadata_index.add_installed(dir_name.clone(), im.file_id, im.clone()).await;
                        installed.push((dir_name, ModDirectory::Nexus(im.clone()).into()));
                        by_file_id.push((im.file_id, im));
                    }
                    Err(_) => {
                        installed.push((dir_name, ModDirectory::Unknown.into()));
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
        }
    }

    pub async fn get(&self, name: &String) -> Option<(usize, String, Arc<ModDirectory>)> {
        self.mods.read().await.get_full(name).map(|(i, k, v)| (i, k.clone(), v.clone()))
    }

    pub async fn add(&self, dir_name: String, md: Arc<ModDirectory>) {
        if let ModDirectory::Nexus(im) = md.as_ref() {
            self.metadata_index.add_installed(dir_name.clone().clone(), im.file_id, im.clone()).await;
        }
        self.mods.write().await.insert(dir_name, md);
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn delete(&self, dir_name: &String) {
        let mut mods_lock = self.mods.write().await;
        if let Some(mod_dir) = mods_lock.swap_remove(dir_name) {
            if let ModDirectory::Nexus(im) = mod_dir.as_ref() {
                let mfd = self
                    .metadata_index
                    .get_by_file_id(&im.file_id)
                    .await
                    .expect(&format!("{} should have been present in the metadata index.", &im.file_id));
                let maybe_unreferenced;
                {
                    let mut mfd_installed_lock = mfd.installed_mods.write().await;
                    mfd_installed_lock.remove(dir_name);
                    maybe_unreferenced = mfd_installed_lock.is_empty();
                }; // drop lock here since delete_if_unreferenced() will need it
                if maybe_unreferenced {
                    self.metadata_index.delete_if_unreferenced(&mfd.file_id).await;
                }
            }
            let path = self.config.install_dir().join(&dir_name);
            if let Err(e) = fs::remove_dir_all(path).await {
                self.logger.log(format!("Error {e} when removing {dir_name}"));
            }
            self.has_changed.store(true, Ordering::Relaxed);
        } else {
            self.logger
                .log(format!("{dir_name} no longer exists. Please file a bug report if you did not just remove it."));
        }
    }
}
