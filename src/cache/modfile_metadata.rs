use crate::api::{FileDetails, ModInfo, UpdateStatus, UpdateStatusWrapper};
use crate::cache::{ArchiveFile, Cacheable};
use crate::config::{Config, DataPath};
use crate::install::{InstalledMod, ModDirectory};
use crate::Logger;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ModFileMetadata {
    pub game: String,
    pub mod_id: u32,
    pub file_id: u64,
    pub file_details: Arc<RwLock<Option<Arc<FileDetails>>>>,
    pub installed_mods: Arc<RwLock<HashMap<String, Arc<InstalledMod>>>>,
    pub mod_info: Arc<RwLock<Option<Arc<ModInfo>>>>,
    pub mod_archives: Arc<RwLock<HashMap<String, Arc<ArchiveFile>>>>,
    pub update_status: UpdateStatusWrapper,
}

impl ModFileMetadata {
    pub fn new(
        game: String,
        mod_id: u32,
        file_id: u64,
        file_details: Option<Arc<FileDetails>>,
        installed_mod: Option<(String, Arc<InstalledMod>)>,
        mod_info: Option<Arc<ModInfo>>,
        mod_archive: Option<Arc<ArchiveFile>>,
    ) -> Self {
        let mut installed_map = HashMap::new();
        if let Some((dir_name, ins_mod)) = installed_mod {
            installed_map.insert(dir_name, ins_mod);
        }

        let update_status = {
            let mut latest_status: UpdateStatus = UpdateStatus::UpToDate(0);
            for (_, ins_mod) in &installed_map {
                let ins_status = ins_mod.update_status.to_enum();
                if latest_status.time() < ins_status.time() {
                    latest_status = ins_status;
                }
            }
            if let Some(archive) = &mod_archive {
                if let Some(metadata) = &archive.mod_data {
                    let archive_status = metadata.update_status.to_enum();
                    if latest_status.time() < archive_status.time() {
                        latest_status = archive_status;
                    }
                }
            }
            latest_status
        }
        .into();

        let mut mod_archives = HashMap::new();
        if let Some(a) = &mod_archive {
            mod_archives.insert(a.file_name.clone(), a.clone());
        }

        Self {
            game,
            mod_id,
            file_id,
            mod_archives: Arc::new(mod_archives.into()),
            file_details: Arc::new(file_details.into()),
            mod_info: Arc::new(mod_info.into()),
            update_status,
            installed_mods: Arc::new(installed_map.into()),
        }
    }

    pub async fn uploaded_timestamp(&self) -> Option<u64> {
        self.file_details.read().await.as_ref().map(|fd| fd.uploaded_timestamp)
    }

    pub async fn propagate_update_status(&self, config: &Config, logger: &Logger, status: &UpdateStatus) {
        self.update_status.set(status.clone());
        for (_, archive) in self.mod_archives.write().await.iter() {
            if let Some(metadata) = &archive.mod_data {
                metadata.update_status.set(status.clone());
                if let Err(e) = metadata.save_changes(DataPath::ArchiveMetadata(&config, &archive.file_name)).await {
                    logger.log(format!("Couldn't save UpdateStatus for {}: {}", archive.file_name, e));
                }
            }
        }
        for (dir_name, installed) in self.installed_mods.write().await.iter() {
            installed.update_status.set(status.clone());
            if let Err(e) = ModDirectory::Nexus(installed.clone()).save_changes(DataPath::ModDirMetadata(&config, dir_name)).await {
                logger.log(format!("Couldn't save UpdateStatus for {}: {}", dir_name, e));
            }
        }
    }

    pub async fn name(&self) -> Option<String> {
        self.file_details.read().await.as_ref().map(|fd| fd.name.clone())
    }

    pub async fn mod_name(&self) -> Option<String> {
        if let Some(mod_name) = self.mod_info.read().await.as_ref().map(|mi| mi.name.clone()).flatten() {
            return Some(mod_name);
        } else {
            for (_, im) in self.installed_mods.read().await.iter() {
                if let Some(mod_name) = &im.mod_name {
                    return Some(mod_name.clone());
                }
            }
        }
        None
    }

    pub async fn file_details(&self) -> Option<Arc<FileDetails>> {
        self.file_details.read().await.as_ref().map(|fd| fd.clone())
    }
}

impl Eq for ModFileMetadata {}
impl PartialEq for ModFileMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.file_id == other.file_id
    }
}
impl Hash for ModFileMetadata {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.file_id.hash(state);
    }
}
