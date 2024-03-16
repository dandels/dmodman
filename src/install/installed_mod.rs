use crate::api::update_status::*;
use crate::cache::{ArchiveFile, ArchiveMetadata, Cacheable};
use crate::Cache;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum ModDirectory {
    Nexus(Arc<InstalledMod>),
    // Installed with dmodman but not a regular Nexus file
    Foreign(String), // archive name
    // Some other directory found in the installs dir
    #[default]
    Unknown,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum ModRepository {
    Nexus,
    #[default]
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstalledMod {
    pub game: String,
    pub mod_id: u32,
    pub file_id: u64,
    pub name: Option<String>,
    pub mod_name: Option<String>,
    pub version: Option<String>,
    pub category_id: Option<u32>,
    pub category_name: Option<String>,
    pub installation_file: String,
    pub repository: ModRepository,
    pub last_update_check: Arc<AtomicU64>,
    pub update_status: UpdateStatusWrapper,
}

impl ModDirectory {
    pub async fn new(cache: Cache, archive: Arc<ArchiveFile>) -> Self {
        if archive.mod_data.is_none() {
            return ModDirectory::Foreign(archive.file_name.clone());
        }
        let ArchiveMetadata {
            game, mod_id, file_id, ..
        } = archive.mod_data.as_ref().unwrap().as_ref();
        let mfd = cache.metadata_index.get_by_archive_name(&archive.file_name).await.unwrap();
        let (version, category_id, category_name, update_status) = {
            if let Some(fd) = mfd.file_details.read().await.as_ref() {
                (fd.version.clone(), Some(fd.category_id), fd.category_name.clone(), Some(mfd.update_status.clone()))
            } else {
                (None, None, None, None) // any other way to do this?
            }
        };
        let update_status = match update_status {
            Some(status) => mfd.update_status.clone().return_later(status),
            None => mfd.update_status.clone(),
        };
        let mod_name = mfd.mod_name().await;
        let name = mfd.name().await;
        ModDirectory::Nexus(
            InstalledMod {
                installation_file: archive.file_name.clone(),
                game: game.clone(),
                mod_id: *mod_id,
                file_id: *file_id,
                name,
                mod_name,
                version,
                category_id,
                category_name,
                repository: ModRepository::Nexus,
                last_update_check: AtomicU64::new(0).into(),
                update_status,
            }
            .into(),
        )
    }
}

impl Cacheable for ModDirectory {}
