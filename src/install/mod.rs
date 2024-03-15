mod install_error;
pub mod installed_mod;

pub use self::install_error::InstallError;
pub use self::installed_mod::*;

use crate::cache::{ArchiveEntry, ArchiveStatus, Cache, Cacheable};
use crate::config::{Config, DataPath};
use crate::Logger;
use async_zip::base::read::WithoutEntry;
use async_zip::error::ZipError;
use async_zip::tokio::read::fs::ZipFileReader;
use async_zip::tokio::read::ZipEntryReader;
use futures_lite::AsyncReadExt as FuturesReadExt;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::RwLock;
use tokio::{task, task::JoinHandle};

#[derive(Clone)]
pub struct Installer {
    cache: Cache,
    config: Arc<Config>,
    logger: Logger,
    pub extract_jobs: Arc<RwLock<HashSet<PathBuf>>>,
    // used by UI to ask if table needs to be redrawn
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

    pub async fn list_content(&self, archive_name: &String) -> Result<Vec<String>, ZipError> {
        let path = self.config.download_dir().join(archive_name);
        task::spawn(async move {
            // TODO only works for .zip files
            let reader = ZipFileReader::new(path).await?;
            let mut entries: Vec<String> = vec![];
            for entry in reader.file().entries() {
                entries.push(entry.filename().clone().into_string()?);
            }
            Ok(entries)
        })
        .await
        .unwrap()
    }

    pub async fn extract(
        &self,
        archive_name: &str,
        dest_dir_name: String,
        overwrite: bool,
    ) -> Result<(), InstallError> {
        let src_path = self.config.download_dir().join(archive_name);
        let mut dest_path = self.config.install_dir();
        dest_path.push(&dest_dir_name);

        let mut jobs = self.extract_jobs.write().await;
        if jobs.contains(&dest_path) {
            return Err(InstallError::InProgress);
        }

        if !overwrite && dest_path.exists() {
            return Err(InstallError::AlreadyExists);
        }

        let archive = match self.cache.archives.get(archive_name).await {
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
        match ZipFileReader::new(src_path).await {
            Ok(zip_reader) => {
                jobs.insert(dest_path.clone());
                drop(jobs);

                *archive.status.write().await = ArchiveStatus::Extracting;
                self.cache.archives.has_changed.store(true, Ordering::Relaxed);
                let mod_dir = ModDirectory::new(self.cache.clone(), archive.clone()).await;
                if let Err(e) = mod_dir.save(DataPath::ModDirMetadata(&self.config, &dest_dir_name)).await {
                    self.logger.log(format!("Failed to save metadata for extracted directory {}, {e}", &dest_dir_name));
                }

                let me = self.clone();
                let archive_name = archive_name.to_string();
                tokio::fs::create_dir_all(&dest_path).await.unwrap();
                let _handle = task::spawn(async move {
                    me.logger.log(format!("Begin extracting: {:?}", &archive_name));
                    let mut tasks: Vec<JoinHandle<Result<usize, (usize, InstallError)>>> = vec![];
                    for (i, entry) in zip_reader.file().entries().iter().enumerate() {
                        let zip_reader = zip_reader.clone();
                        let logger = me.logger.clone();
                        let base_dir = dest_path.clone();
                        let file_name = entry.filename().clone().into_string().unwrap();
                        match entry.dir() {
                            Ok(is_dir) => {
                                if is_dir {
                                    // TODO path traversal vulnerability, this needs to be sanitized
                                    if let Err(e) = tokio::fs::create_dir_all(base_dir.join(&file_name)).await {
                                        me.logger.log(format!("Failed to extract directory {file_name}: {e}"));
                                        let mut jobs = me.extract_jobs.write().await;
                                        jobs.remove(&dest_path);
                                        return Err(e.into());
                                    }
                                } else {
                                    tasks.push(task::spawn(async move {
                                        extract_zip_entry(logger, base_dir, file_name, zip_reader, i).await
                                    }));
                                }
                            }
                            Err(e) => {
                                me.logger.log(format!("Failed to read archive file, aborting: {e}"));
                            }
                        }
                    }

                    for task in tasks {
                        if let Err((_index, e)) = task.await.unwrap() {
                            me.logger.log(format!("WARN: Extraction failed with error: {e}.",));
                            // TODO figure out accurate installation status
                            *archive.status.write().await = ArchiveStatus::Downloaded;

                            let mut jobs = me.extract_jobs.write().await;
                            jobs.remove(&dest_path);
                            return Err(e);
                        }
                    }
                    *archive.status.write().await = ArchiveStatus::Installed;
                    me.cache.archives.has_changed.store(true, Ordering::Relaxed);
                    let mut jobs = me.extract_jobs.write().await;
                    jobs.remove(&dest_path);
                    me.cache.installed.add(dest_dir_name, mod_dir).await;
                    me.logger.log(format!("Finished extracting: {:?}", &archive_name));
                    Ok(())
                });
            }
            Err(e) => {
                self.logger.log(format!("Unable to extract: {archive_name}"));
                self.logger.log(format!("{:?}", e));
            }
        }
        Ok(())
    }
}

async fn extract_zip_entry(
    logger: Logger,
    base_dir: PathBuf,
    file_name: String,
    reader: ZipFileReader,
    index: usize,
) -> Result<usize, (usize, InstallError)> {
    // TODO Sanitize path
    // this fails because canonicalize() checks if it exists
    //let path = base_dir.join(match PathBuf::from(&file_name).canonicalize() {
    //    Ok(p) => p,
    //    Err(e) => {
    //        logger.log(format!("File name was {:?}, err {:?}", &file_name, &e));
    //        return Err((index, e.into()));
    //    }
    //});
    let path = base_dir.join(&file_name);
    let mut entry_reader: ZipEntryReader<File, WithoutEntry> = reader.reader_without_entry(index).await.unwrap();
    match File::create(path.clone()).await {
        Ok(file) => {
            // TODO is it possible and desirable to use a BufReader for the zip file? The trait bounds are a bit crazy
            // though.
            let mut writer = BufWriter::new(file);
            let mut buf = [0; 4096];
            loop {
                match entry_reader.read(&mut buf).await {
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            writer.flush().await.unwrap();
                            return Ok(index);
                        }
                        if let Err(e) = writer.write(&mut buf[..bytes_read]).await {
                            logger.log(format!("Writing {:?} reported error {e}", &path));
                            logger.log(format!("{e}"));
                            return Err((index, e.into()));
                        }
                    }
                    Err(e) => {
                        logger.log(format!("Extracting {:?} failed with {e}", &path));
                        return Err((index, e.into()));
                    }
                }
            }
        }
        Err(e) => {
            logger.log(format!("Failed to create file {:?}", &path));
            return Err((index, e.into()));
        }
    }
}
