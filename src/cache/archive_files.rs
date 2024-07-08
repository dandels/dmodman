use super::MetadataIndex;
use crate::api::downloads::FileInfo;
use crate::api::update_status::*;
use crate::cache::{Cacheable, Installed};
use crate::config::Config;
use crate::Logger;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tokio::fs;
use tokio::fs::File;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ArchiveFiles {
    config: Arc<Config>,
    logger: Logger,
    metadata_index: MetadataIndex,
    pub files: Arc<RwLock<IndexMap<String, ArchiveEntry>>>, // indexed by name
    pub has_changed: Arc<AtomicBool>,
}

impl ArchiveFiles {
    pub async fn new(
        config: Arc<Config>,
        logger: Logger,
        installed: Installed,
        file_index: MetadataIndex,
        has_changed: Arc<AtomicBool>,
    ) -> Self {
        // TODO fix error handling here
        std::fs::create_dir_all(config.download_dir()).unwrap();

        /* Sort files by creation time.
         * This is easier with std::fs and we always block on Cache initialization anyway. */
        let mut dir_entries: Vec<_> = match std::fs::read_dir(config.download_dir()) {
            Ok(rd) => rd.map(|f| f.unwrap()).collect(),
            Err(_) => vec![],
        };
        dir_entries.sort_by_key(|f| match f.metadata() {
            Ok(md) => md.created().unwrap(),
            Err(_) => UNIX_EPOCH,
        });

        let mut files = IndexMap::new();

        for f in dir_entries {
            let path = f.path();
            let file_ext = path.extension().and_then(OsStr::to_str);
            // Skip .part and .part.json files
            if !path.is_file() || file_ext == Some("part") || path.ends_with(".part.json") {
                continue;
            }
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            // Only .json file for archive is present
            if file_ext == Some("json") && !path.with_extension("").exists() {
                match ArchiveMetadata::load(path).await {
                    Ok(md) => {
                        let entry = ArchiveEntry::MetadataOnly(Arc::new(md));
                        file_index.try_add_mod_archive(entry.clone()).await;
                        files.insert(entry.file_name().clone(), entry);
                    }
                    Err(e) => {
                        logger.log(format!("Failed to deserialize {} as archive metadata: {e}", file_name));
                    }
                };
            // Archive exists, might also have .json file
            } else if file_ext != Some("json") {
                let json_file = path.with_file_name(format!("{}.json", file_name));
                let mod_data = match ArchiveMetadata::load(json_file).await {
                    Ok(md) => Some(Arc::new(md)),
                    Err(e) => {
                        // Only log error if it's for some other reason than NotFound
                        if e.kind() != std::io::ErrorKind::NotFound {
                            logger.log(format!("{} is missing its metadata: {e}", file_name));
                        }
                        None
                    }
                };
                if let Ok(af) = ArchiveFile::new(&logger, &installed, &path, mod_data).await {
                    let entry = ArchiveEntry::File(Arc::new(af));
                    file_index.try_add_mod_archive(entry.clone()).await;
                    files.insert(entry.file_name().clone(), entry);
                }
            }
        }
        Self {
            config,
            logger,
            metadata_index: file_index,
            files: Arc::new(RwLock::new(files)),
            has_changed,
        }
    }

    pub async fn add_archive(&self, archive: ArchiveEntry) {
        self.metadata_index.try_add_mod_archive(archive.clone()).await;
        self.files.write().await.insert(archive.file_name().clone(), archive);
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn get(&self, file_name: &str) -> Option<ArchiveEntry> {
        self.files.read().await.get(file_name).cloned()
    }

    pub async fn delete(&self, file_name: &str) {
        let mut lock = self.files.write().await;
        if let Some(_archive_file) = lock.get(file_name) {
            let path = self.config.download_dir().join(file_name);
            if path.exists() {
                if let Err(e) = fs::remove_file(&path).await {
                    self.logger.log(format!("Error when removing file: {e}"));
                    return;
                }
            }
            let json_file_name = path.file_name().unwrap().to_string_lossy();
            let json_file = path.with_file_name(format!("{}.json", json_file_name));
            if json_file.exists() {
                if let Err(e) = fs::remove_file(&json_file).await {
                    self.logger.log(format!("Error when removing file: {e}"));
                    return;
                }
            }
            lock.shift_remove(file_name);
            self.has_changed.store(true, Ordering::Relaxed);
        }
    }
}

pub struct ArchiveFile {
    pub file_name: String,
    pub size: u64,
    pub mod_data: Option<Arc<ArchiveMetadata>>,
    pub install_state: Arc<RwLock<ArchiveStatus>>,
}

impl ArchiveFile {
    pub async fn new(
        logger: &Logger,
        installed: &Installed,
        path: &PathBuf,
        mod_data: Option<Arc<ArchiveMetadata>>,
    ) -> Result<Self, std::io::Error> {
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let size: u64 = match File::open(path).await {
            Ok(file) => match file.metadata().await {
                Ok(md) => md.len(),
                Err(e) => {
                    logger.log(format!("Unable to get file metadata of {}: {e}", file_name));
                    return Err(e);
                }
            },
            Err(e) => {
                logger.log(format!("Unable to open {} for reading its metadata: {e}", file_name));
                return Err(e);
            }
        };

        let install_status = {
            if let Some(md) = &mod_data {
                match installed.get(&md.file_name).await {
                    Some(_) => ArchiveStatus::Installed,
                    None => ArchiveStatus::Downloaded,
                }
            } else {
                ArchiveStatus::Downloaded
            }
        };

        Ok(Self {
            file_name,
            size,
            mod_data,
            install_state: Arc::new(install_status.into()),
        })
    }
}

#[derive(Clone)]
pub enum ArchiveEntry {
    File(Arc<ArchiveFile>),
    MetadataOnly(Arc<ArchiveMetadata>),
}

impl ArchiveEntry {
    pub fn file_name(&self) -> &String {
        match self {
            ArchiveEntry::File(archive) => &archive.file_name,
            ArchiveEntry::MetadataOnly(metadata) => &metadata.file_name,
        }
    }

    pub fn metadata(&self) -> Option<Arc<ArchiveMetadata>> {
        match self {
            ArchiveEntry::File(archive) => archive.mod_data.clone(),
            ArchiveEntry::MetadataOnly(metadata) => Some(metadata.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ArchiveStatus {
    Downloaded,
    Extracting,
    Error,
    Installed,
}

impl Display for ArchiveStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveStatus::Downloaded => f.write_str(""),
            ArchiveStatus::Extracting => f.write_str("Extracting"),
            ArchiveStatus::Error => f.write_str("Error"),
            ArchiveStatus::Installed => f.write_str("Installed"),
        }
    }
}

impl Hash for ArchiveFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.file_name.hash(state)
    }
}

impl Eq for ArchiveFile {}
impl PartialEq for ArchiveFile {
    fn eq(&self, other: &Self) -> bool {
        self.file_name.eq(&other.file_name)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ArchiveMetadata {
    pub file_name: String,
    pub game: String,
    pub mod_id: u32,
    pub file_id: u64,
    pub update_status: UpdateStatusWrapper,
}

impl ArchiveMetadata {
    pub fn new(fi: FileInfo, update_status: UpdateStatus) -> Self {
        ArchiveMetadata {
            game: fi.game,
            file_name: fi.file_name,
            mod_id: fi.mod_id,
            file_id: fi.file_id,
            update_status: UpdateStatusWrapper::new(update_status),
        }
    }
}

impl Cacheable for ArchiveMetadata {}
