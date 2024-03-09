use super::MetadataIndex;
use crate::api::downloads::FileInfo;
use crate::api::update_status::*;
use crate::cache::{Cacheable, Installed};
use crate::config::Config;
use crate::install::InstallStatus;
use crate::Logger;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tokio::fs::File;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ArchiveFiles {
    #[allow(dead_code)]
    config: Config,
    #[allow(dead_code)]
    logger: Logger,
    #[allow(dead_code)]
    file_index: MetadataIndex,
    pub files: Arc<RwLock<IndexMap<String, Arc<ArchiveFile>>>>, // indexed by name
    pub has_changed: Arc<AtomicBool>,
}

impl ArchiveFiles {
    pub async fn new(config: Config, logger: Logger, installed: Installed, file_index: MetadataIndex) -> Self {
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
            // Skip .part and .json files
            if path.is_file() && ![Some("json"), Some("part")].contains(&file_ext) {
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let json_file = path.with_file_name(format!("{}.json", file_name));
                let mod_data = match ArchiveMetadata::load(json_file).await {
                    Ok(md) => Some(Arc::new(md)),
                    Err(e) => {
                        logger.log(format!("{} is missing its metadata: {e}", file_name));
                        None
                    }
                };
                if let Some(af) = ArchiveFile::new(&logger, &installed, &path, mod_data).await {
                    let af = Arc::new(af);
                    file_index.try_add_mod_archive(af.clone()).await;
                    files.insert(af.file_name.clone(), af);
                }
            }
        }
        Self {
            config,
            logger,
            file_index,
            files: Arc::new(RwLock::new(files)),
            has_changed: Arc::new(true.into()),
        }
    }

    pub async fn add(&self, archive: Arc<ArchiveFile>) {
        self.file_index.try_add_mod_archive(archive.clone()).await;
        self.files.write().await.insert(archive.file_name.clone(), archive);
    }

    pub async fn get(&self, file_name: &String) -> Option<Arc<ArchiveFile>> {
        self.files.read().await.get(file_name).cloned()
    }

    pub async fn get_by_index(&self, index: usize) -> Option<Arc<ArchiveFile>> {
        self.files.read().await.get_index(index).map(|(_, af)| af.clone())
    }
}

pub struct ArchiveFile {
    pub file_name: String,
    pub size: u64,
    pub mod_data: Option<Arc<ArchiveMetadata>>,
    pub install_status: Arc<RwLock<InstallStatus>>,
}

impl ArchiveFile {
    pub async fn new(
        logger: &Logger,
        installed: &Installed,
        path: &PathBuf,
        mod_data: Option<Arc<ArchiveMetadata>>,
    ) -> Option<Self> {
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
        let size: u64 = match File::open(path).await {
            Ok(file) => match file.metadata().await {
                Ok(md) => md.len(),
                Err(e) => {
                    logger.log(format!("Unable to get file metadata of {}: {e}", file_name));
                    return None;
                }
            },
            Err(e) => {
                logger.log(format!("Unable to open {} for reading its metadata: {e}", file_name));
                return None;
            }
        };

        let install_status = {
            if let Some(md) = &mod_data {
                match installed.by_file_id.read().await.get(&md.file_id) {
                    Some(_) => InstallStatus::Installed,
                    None => InstallStatus::Downloaded,
                }
            } else {
                InstallStatus::Downloaded
            }
        };

        Some(Self {
            file_name,
            size,
            mod_data,
            install_status: Arc::new(install_status.into()),
        })
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
