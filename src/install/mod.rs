mod install_error;
pub mod installed_mod;
mod libarchive;

pub use self::install_error::InstallError;
pub use self::installed_mod::*;

use crate::cache::{ArchiveEntry, ArchiveFile, ArchiveStatus, Cache, Cacheable};
use crate::config::{Config, DataPath};
use crate::Logger;
use libarchive::*;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::task;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct Installer {
    cache: Cache,
    config: Arc<Config>,
    logger: Logger,
    pub extract_jobs: Arc<RwLock<HashMap<String, CancellationToken>>>, // key: archive name
}
impl Installer {
    pub async fn new(cache: Cache, config: Arc<Config>, logger: Logger) -> Self {
        Self {
            cache,
            config,
            logger,
            extract_jobs: Default::default(),
        }
    }

    pub async fn list_content(&self, archive_name: &str) -> Option<Result<Vec<String>, InstallError>> {
        if let Some(ArchiveEntry::File(archive)) = self.cache.archives.get(archive_name).await {
            let path = self.config.download_dir().join(&archive.file_name);
            return Some({
                let task: Result<Vec<String>, InstallError> = task::spawn(async move {
                    let mut files = Vec::new();
                    let archive = Archive::open(path.to_string_lossy().to_string()).await?;
                    while let Some(entry_res) = archive.next().await {
                        match entry_res {
                            Ok(entry) => {
                                if !entry.is_dir().await {
                                    files.push(entry.path().await.to_string_lossy().to_string());
                                }
                            }
                            Err(e) => return Err(e.into()),
                        }
                    }
                    Ok(files)
                })
                .await
                .unwrap();
                task
            });
        }
        None
    }

    pub async fn extract(
        &self,
        archive_name: String,
        dest_dir_name: String,
        overwrite: bool,
    ) -> Result<(), InstallError> {
        let src_path = self.config.download_dir().join(&archive_name);
        let mut dest_path = self.config.install_dir();
        dest_path.push(&dest_dir_name);

        if !overwrite && dest_path.exists() {
            return Err(InstallError::AlreadyExists);
        }

        let archive_file = match self.cache.archives.get(&archive_name).await {
            Some(entry) => match entry {
                ArchiveEntry::File(archive) => archive,
                ArchiveEntry::MetadataOnly(_) => {
                    self.logger.log(format!("Archive {} no longer exists.", archive_name));
                    return Err(InstallError::ArchiveDeleted);
                }
            },
            /* Race condition between UI and backend if backend starts removing entries based on inotify events.
             * unlikely to actually happen */
            None => {
                self.logger.log(format!("{} no longer exists in the database..?", archive_name));
                return Ok(());
            }
        };

        let mut jobs = self.extract_jobs.write().await;
        if jobs.contains_key(&archive_name) {
            return Err(InstallError::InProgress);
        }

        let me = self.clone();
        let target_dir_name = dest_dir_name.clone();
        let cancel_token = CancellationToken::new();
        let cloned_token = cancel_token.clone();
        jobs.insert(archive_name.clone(), cancel_token);
        task::spawn(async move {
            /* The select macro runs both futures at once, then allows us to run a function after the first one
             * finishes. It's well suited for cancelling a task and cleaning up afterwards.
             */
            tokio::select! {
                _ = cloned_token.cancelled() => {
                    if let Err(e) = fs::remove_dir_all(dest_path).await {
                        me.logger.log(format!("Unable to remove target directory: {e}"));
                    }
                },
                _ = async {
                    let mod_dir = me.pre_extract(archive_file.clone(), &dest_path, &target_dir_name).await;

                    let res: Result<(), InstallError> = match Archive::open(src_path.to_string_lossy().to_string()).await {
                        Ok(archive) => {
                            //let mut reader = AsyncArchiveReader::new(archive);
                            while let Some(entry_res) = archive.next().await {
                                let entry = {
                                    if let Err(e) = entry_res {
                                        me.logger.log(format!("Unable to get next archive entry: {e}."));
                                        let err: InstallError = e.into();
                                        return Err(err);
                                    }
                                    entry_res.unwrap()
                                };
                                let target_path = dest_path.join(normalize_path(&entry.path().await));
                                let name = target_path.file_name().unwrap().to_string_lossy();
                                crate::logger::log_to_file(format!("Archive: {name}"));
                                if entry.is_dir().await {
                                    drop(entry);
                                    if let Err(e) = fs::create_dir_all(&target_path).await {
                                        me.logger.log(format!("Failed to extract directory {name}: {e}"));
                                        let mut jobs = me.extract_jobs.write().await;
                                        jobs.remove(&target_dir_name);
                                        return Err(e.into());
                                    }
                                } else {
                                    let parent = target_path.parent().unwrap_or(&dest_path);
                                    if !parent.exists() {
                                        std::fs::create_dir_all(parent).unwrap();
                                    }
                                    extract_entry(me.logger.clone(), target_path, archive.clone()).await.unwrap();
                                    crate::logger::log_to_file("Done with first file...?");
                                }
                            }
                            Ok(())
                        }
                        Err(e) => Err(e.into()),
                    };

                    if res.is_ok() {
                        me.post_extract(archive_file, target_dir_name, mod_dir).await;
                    } else {
                        *archive_file.install_state.write().await = ArchiveStatus::Error;
                        me.cache.archives.has_changed.store(true, Ordering::Relaxed);
                        // TODO maybe clean up after a failed extraction?
                        me.logger.log(format!("Aborted extracting \"{}\". Output directory has not been removed.", archive_name));
                        me.extract_jobs.write().await.remove(&target_dir_name);
                    }

                    Ok(())
                } => {
                }
            }
        });
        Ok(())
    }

    pub async fn cancel(&self, archive: &ArchiveFile) {
        if let Some(token) = self.extract_jobs.write().await.remove(&archive.file_name) {
            token.cancel();
            if let Some(mfd) = self.cache.metadata_index.get_by_archive_name(&archive.file_name).await {
                if mfd.is_installed().await {
                    *archive.install_state.write().await = ArchiveStatus::Installed;
                    self.cache.archives.has_changed.store(true, Ordering::Relaxed);
                    return;
                }
            }
            *archive.install_state.write().await = ArchiveStatus::Downloaded;
            self.cache.archives.has_changed.store(true, Ordering::Relaxed);
        }
    }

    async fn pre_extract(
        &self,
        archive: Arc<ArchiveFile>,
        dest_path: &PathBuf,
        dest_dir_name: &String,
    ) -> ModDirectory {
        self.logger.log(format!("Begin extracting: {:?}", &archive.file_name));
        *archive.install_state.write().await = ArchiveStatus::Extracting;
        fs::create_dir_all(&dest_path).await.unwrap();
        self.cache.archives.has_changed.store(true, Ordering::Relaxed);
        let mod_dir = ModDirectory::new(self.cache.clone(), archive.clone()).await;
        if let Err(e) = mod_dir.save(DataPath::ModDirMetadata(&self.config, dest_dir_name)).await {
            self.logger.log(format!("Failed to save metadata for extracted directory {}, {e}", dest_dir_name));
        }
        mod_dir
    }

    async fn post_extract(&self, archive: Arc<ArchiveFile>, dest_dir_name: String, mod_dir: ModDirectory) {
        *archive.install_state.write().await = ArchiveStatus::Installed;
        self.cache.archives.has_changed.store(true, Ordering::Relaxed);
        self.cache.installed.add(dest_dir_name.clone(), mod_dir).await;
        self.extract_jobs.write().await.remove(&archive.file_name);
        self.logger.log(format!("Finished extracting: {:?}", &archive.file_name));
    }
}

async fn extract_entry(logger: Logger, target_path: PathBuf, archive: Archive) -> Result<(), InstallError> {
    match File::create(&target_path).await {
        Ok(mut file) => {
            loop {
                let (status, bytes) = archive.read_data_block().await;
                match status {
                    bindings::ARCHIVE_OK | bindings::ARCHIVE_WARN => {
                        //crate::logger::log_to_file(format!("Got {} bytes", bytes.len()));
                        if let bindings::ARCHIVE_WARN = status {
                            logger.log(format!(
                                "Warning when extracting \"{:?}\": {}",
                                &target_path,
                                archive.get_err_msg().await
                            ));
                        }
                        if let Some(bytes) = bytes {
                            if let Err(e) = file.write_all(&bytes).await {
                                logger.log(format!("Writing {:?} reported error {e}", &target_path));
                                return Err(e.into());
                            }
                        }
                    }
                    bindings::ARCHIVE_EOF => {
                        crate::logger::log_to_file("EOF!".to_string());
                        return Ok(());
                    }
                    // TODO handle ARCHIVE_RETRY
                    _ => {
                        let msg = archive.get_err_msg().await;
                        logger.log(format!("Error when extracting \"{:?}\": {}", &target_path, &msg));
                        return Err(ArchiveError::from_err_code(status, msg).into());
                    }
                }
            }
        }
        Err(e) => {
            logger.log(format!("Failed to create file {:?}", &target_path));
            Err(e.into())
        }
    }
}

/* The standard library does unfortunately not offer a way to normalize a path.
 * path.canonicalize() checks if the file exists, which is not what we want here.
 *
 * Credits to the Cargo project (MIT licensed), particularly Alex Crichton who seems to have written this.
 * https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61 */
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}
