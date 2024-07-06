use crate::api::downloads::FileInfo;
use crate::api::{FileDetails, ModInfo, UpdateStatus, UpdateStatusWrapper};
use crate::cache::{ArchiveFile, ArchiveStatus, Cacheable};
use crate::config::{Config, DataPath};
use crate::extract::{InstalledMod, ModDirectory};
use crate::Logger;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ModFileMetadata {
    pub game: String,
    pub mod_id: u32,
    pub file_id: u64,
    file_details: Arc<RwLock<Option<Arc<FileDetails>>>>,
    installed_mods: Arc<RwLock<HashMap<String, Arc<InstalledMod>>>>,
    mod_archives: Arc<RwLock<HashMap<String, Arc<ArchiveFile>>>>,
    pub mod_info: Arc<RwLock<Option<Arc<ModInfo>>>>,
    pub update_status: UpdateStatusWrapper,
}

impl From<&InstalledMod> for ModFileMetadata {
    fn from(im: &InstalledMod) -> Self {
        Self::new(im.game.clone(), im.mod_id, im.file_id)
    }
}

impl From<&FileInfo> for ModFileMetadata {
    fn from(fi: &FileInfo) -> Self {
        Self::new(fi.game.clone(), fi.mod_id, fi.file_id)
    }
}

impl ModFileMetadata {
    pub fn new(game: String, mod_id: u32, file_id: u64) -> Self {
        Self {
            game,
            mod_id,
            file_id,
            mod_archives: Default::default(),
            file_details: Default::default(),
            mod_info: Default::default(),
            update_status: Default::default(),
            installed_mods: Default::default(),
        }
    }

    pub async fn add_archive(&self, archive: Arc<ArchiveFile>) {
        if let Some(md) = &archive.mod_data {
            self.update_status.sync_with(&md.update_status);
        }
        if !self.installed_mods.read().await.is_empty() {
            *archive.install_state.write().await = ArchiveStatus::Installed;
        }
        self.mod_archives.write().await.insert(archive.file_name.clone(), archive);
    }

    pub async fn add_installed_dir(&self, dir_name: String, mod_dir: Arc<InstalledMod>) {
        for (_, archive) in self.mod_archives.write().await.iter() {
            *archive.install_state.write().await = ArchiveStatus::Installed;
        }
        self.update_status.sync_with(&mod_dir.update_status);
        self.installed_mods.write().await.insert(dir_name, mod_dir);
    }

    pub async fn file_details(&self) -> Option<Arc<FileDetails>> {
        self.file_details.read().await.clone()
    }

    pub async fn is_installed(&self) -> bool {
        !self.installed_mods.read().await.is_empty()
    }

    pub async fn is_unreferenced(&self) -> bool {
        self.installed_mods.read().await.is_empty() && self.mod_archives.read().await.is_empty()
    }

    pub async fn name(&self) -> Option<String> {
        self.file_details.read().await.as_ref().map(|fd| fd.name.clone())
    }

    pub async fn mod_info(&self) -> Option<Arc<ModInfo>> {
        self.mod_info.read().await.clone()
    }

    pub async fn mod_name(&self) -> Option<String> {
        if let Some(mod_name) = self.mod_info.read().await.as_ref().and_then(|mi| mi.name.clone()) {
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

    pub async fn propagate_update_status(&self, config: &Config, logger: &Logger, status: &UpdateStatus) {
        self.update_status.set(status.clone());
        for (_, archive) in self.mod_archives.write().await.iter() {
            if let Some(metadata) = &archive.mod_data {
                metadata.update_status.set(status.clone());
                if let Err(e) = metadata.save(DataPath::ArchiveMetadata(config, &archive.file_name)).await {
                    logger.log(format!("Couldn't save UpdateStatus for {}: {}", archive.file_name, e));
                }
            }
        }
        for (dir_name, installed) in self.installed_mods.write().await.iter() {
            installed.update_status.set(status.clone());
            if let Err(e) =
                ModDirectory::Nexus(installed.clone()).save(DataPath::ModDirMetadata(config, dir_name)).await
            {
                logger.log(format!("Couldn't save UpdateStatus for {}: {}", dir_name, e));
            }
        }
    }

    // Returns whether archives need refresh
    pub async fn remove_installed(&self, dir_name: &str) -> bool {
        let not_installed = {
            let mut im_lock = self.installed_mods.write().await;
            im_lock.remove(dir_name);
            im_lock.is_empty()
        };

        let arch_lock = self.mod_archives.read().await;
        let archives_have_changed = !arch_lock.is_empty();

        if not_installed {
            for archive in arch_lock.values() {
                *archive.install_state.write().await = ArchiveStatus::Downloaded;
            }
        }
        archives_have_changed
    }

    pub async fn set_file_details(&self, fd: Arc<FileDetails>) {
        *self.file_details.write().await = Some(fd);
    }

    pub async fn uploaded_timestamp(&self) -> Option<u64> {
        self.file_details.read().await.as_ref().map(|fd| fd.uploaded_timestamp)
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
