use crate::api::update_status::*;
use crate::cache::{ArchiveFile, Cacheable, ModFileMetadata};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

pub enum ModDirectory {
    Nexus(Arc<InstalledMod>),
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ModRepository {
    Nexus,
    Unknown,
}

#[derive(Default, Debug, Deserialize, Serialize)]
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

impl InstalledMod {
    pub async fn new(archive_info: &ArchiveFile, mod_file_data: &Option<Arc<ModFileMetadata>>) -> Self {
        match &archive_info.mod_data {
            Some(metadata) => {
                let (version, category_id, category_name, update_status_opt) = match mod_file_data {
                    Some(mfd) => mfd
                        .file_details
                        .read()
                        .await
                        .as_ref()
                        .map(|fd| {
                            (
                                fd.version.clone(),
                                Some(fd.category_id),
                                fd.category_name.clone(),
                                Some(mfd.update_status.clone()),
                            )
                        })
                        .unwrap_or_default(),
                    None => Default::default(),
                };
                let update_status = match update_status_opt {
                    Some(status) => match mod_file_data {
                        Some(mfd) => mfd.update_status.clone().return_later(status),
                        None => status,
                    },
                    None => {
                        archive_info.mod_data.as_ref().and_then(|md| Some(md.update_status.clone())).unwrap_or_default()
                    }
                }
                .into();
                let mod_name = match mod_file_data {
                    Some(mfd) => mfd.md5results.read().await.as_ref().map(|res| res.r#mod.name.clone()).flatten(),
                    None => None,
                };
                let name = if let Some(mfd) = mod_file_data {
                    mfd.file_details.read().await.as_ref().and_then(|fd| Some(fd.name.clone()))
                } else {
                    None
                };
                Self {
                    installation_file: archive_info.file_name.clone(),
                    game: metadata.game.clone(),
                    mod_id: metadata.mod_id,
                    file_id: metadata.file_id,
                    name,
                    mod_name,
                    version,
                    category_id,
                    category_name,
                    repository: ModRepository::Nexus,
                    last_update_check: AtomicU64::new(0).into(),
                    update_status,
                }
            }
            None => Self {
                installation_file: archive_info.file_name.clone(),
                update_status: mod_file_data
                    .as_ref()
                    .and_then(|mfd| Some(mfd.update_status.clone()))
                    .unwrap_or_default(),
                ..Default::default()
            },
        }
    }
}

impl Cacheable for InstalledMod {}

impl Default for ModDirectory {
    fn default() -> Self {
        ModDirectory::Unknown
    }
}

impl Default for ModRepository {
    fn default() -> Self {
        ModRepository::Unknown
    }
}
