use crate::cache::{Cacheable, MetadataIndex};
use crate::install::installed_mod::*;
use crate::Config;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Installed {
    #[allow(dead_code)]
    config: Config,
    metadata_index: MetadataIndex,
    pub mods: Arc<RwLock<IndexMap<String, Arc<ModDirectory>>>>, // Key = Directory name
    pub by_file_id: Arc<RwLock<HashMap<u64, Arc<InstalledMod>>>>, // Key = Directory name
    pub has_changed: Arc<AtomicBool>,
}

impl Installed {
    pub async fn new(config: Config, metadata_index: MetadataIndex) -> Self {
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
            metadata_index,
            mods: Arc::new(IndexMap::from_iter(installed).into()),
            by_file_id: Arc::new(HashMap::from_iter(by_file_id).into()),
            has_changed: Arc::new(true.into()),
        }
    }

    #[allow(dead_code)]
    pub async fn get(&self, name: &String) -> Option<(usize, String, Arc<ModDirectory>)> {
        self.mods.read().await.get_full(name).map(|(i, k, v)| (i, k.clone(), v.clone()))
    }

    pub async fn get_by_index(&self, index: usize) -> Option<(String, Arc<ModDirectory>)> {
        self.mods.read().await.get_index(index).map(|(k, v)| (k.clone(), v.clone()))
    }

    pub async fn add(&self, dir_name: String, md: Arc<ModDirectory>) {
        if let ModDirectory::Nexus(im) = md.as_ref() {
            self.metadata_index.add_installed(dir_name.clone().clone(), im.file_id, im.clone()).await;
            self.by_file_id.write().await.insert(im.file_id, im.clone());
        }
        self.mods.write().await.insert(dir_name, md);
        self.has_changed.store(true, Ordering::Relaxed);
    }
}
