mod install_error;
pub mod installed_mod;

pub use self::install_error::InstallError;
pub use self::installed_mod::*;

use crate::cache::{Cache, Cacheable};
use crate::config::{Config, DataType};
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
    config: Config,
    logger: Logger,
    pub extract_jobs: Arc<RwLock<HashSet<PathBuf>>>,
    // used by UI to ask if table needs to be redrawn
}

impl Installer {
    pub async fn new(cache: Cache, config: Config, logger: Logger) -> Self {
        Self {
            cache,
            config,
            logger,
            extract_jobs: Default::default(),
        }
    }

    pub async fn list_content(&self, archive_name: &String) -> Result<Vec<String>, ZipError> {
        let path = self.config.download_dir().join(archive_name);
        //self.logger.log(format!("{path:?}"));
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
        archive_name: String,
        dest_dir_name: String,
        overwrite: bool,
    ) -> Result<(), InstallError> {
        let src_path = self.config.download_dir().join(&archive_name);
        let mut dest_path = self.config.install_dir();
        dest_path.push(&dest_dir_name);

        let mut jobs = self.extract_jobs.write().await;
        if jobs.contains(&dest_path) {
            return Err(InstallError::InProgress);
        }

        if !overwrite && dest_path.exists() {
            return Err(InstallError::AlreadyExists);
        }

        let archive_file = match self.cache.archives.get(&archive_name).await {
            Some(af) => af,
            None => {
                self.logger.log(format!("Metadata for {} was deleted after starting extraction..?", &archive_name));
                return Ok(());
            }
        };

        match ZipFileReader::new(src_path).await {
            Ok(zip_reader) => {
                let mut mfd = None;
                let im = match &archive_file.mod_data {
                    Some(metadata) => {
                        mfd = self.cache.file_index.get_by_file_id(&metadata.file_id).await;
                        if let Some(mfd) = &mfd {
                            // TODO this approach doesn't work for files lacking mfd
                            *mfd.install_status.write().await = InstallStatus::Extracting;
                            self.cache.archives.has_changed.store(true, Ordering::Relaxed);
                        }
                        InstalledMod::new(&archive_file, &mfd).await
                    }
                    None => InstalledMod::new(&archive_file, &None).await,
                };
                if let Err(e) = im.save(self.config.path_for(DataType::InstalledMod(&dest_dir_name))).await {
                    self.logger.log(format!("Failed to save metadata for extracted directory {}, {e}", &dest_dir_name));
                }

                jobs.insert(dest_path.clone());

                let me = self.clone();
                let _handle = task::spawn(async move {
                    me.logger.log(format!("Begin extracting: {:?}", &archive_name));
                    let mut tasks: Vec<JoinHandle<Result<usize, (usize, InstallError)>>> = vec![];
                    for (i, entry) in zip_reader.file().entries().iter().enumerate() {
                        let zip_reader = zip_reader.clone();
                        let logger = me.logger.clone();
                        let base_dir = dest_path.clone();
                        let file_name = entry.filename().clone().into_string().unwrap();
                        tokio::fs::create_dir_all(&dest_path).await.unwrap();
                        tasks.push(task::spawn(async move {
                            extract_zip_entry(logger, base_dir, file_name, zip_reader, i).await
                        }));
                    }

                    for task in tasks {
                        if let Err((_index, e)) = task.await.unwrap() {
                            me.logger
                                .log(format!("WARN: Extract failed with error: {e}, the output might be incomplete.",));
                            if let Some(mfd) = &mfd {
                                // TODO This is only correct if the mod hasn't been installed to some other dir
                                *mfd.install_status.write().await = InstallStatus::Downloaded;
                            }
                            return Err(e);
                        }
                    }
                    me.logger.log(format!("Finished extracting: {:?}", &archive_name));
                    if let Some(mfd) = &mfd {
                        *mfd.install_status.write().await = InstallStatus::Installed;
                    }
                    me.cache.archives.has_changed.store(true, Ordering::Relaxed);
                    let mut jobs = me.extract_jobs.write().await;
                    jobs.remove(&dest_path);
                    me.cache.installed.add(dest_dir_name, Arc::new(ModDirectory::Nexus(im.into()))).await;
                    Ok(())
                });
            }
            Err(e) => {
                //self.logger.log(format!("Unable to extract: {src_path:?}"));
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
