mod install_error;

pub use self::install_error::InstallError;

use crate::config::Config;
use crate::logger::Logger;
use compress_tools::{list_archive_files, uncompress_archive, Ownership};
use indexmap::IndexSet;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::fs;
use tokio::fs::DirEntry;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct Archives {
    config: Config,
    logger: Logger,
    pub has_changed: Arc<AtomicBool>, // used by UI to ask if error list needs to be redrawn
    pub files: Arc<RwLock<Vec<DirEntry>>>,
    pub extract_jobs: Arc<std::sync::RwLock<IndexSet<PathBuf>>>,
}

impl Archives {
    pub fn new(config: Config, logger: Logger) -> Self {
        Self {
            config,
            logger,
            has_changed: AtomicBool::new(true).into(),
            files: Default::default(),
            extract_jobs: Default::default(),
        }
    }

    pub async fn update_list(&self) {
        let mut files: Vec<DirEntry> = vec![];
        if let Ok(mut dir_entries) = fs::read_dir(&self.config.download_dir()).await {
            while let Ok(Some(f)) = dir_entries.next_entry().await {
                let path = f.path();
                if path.is_file() {
                    // TODO more rigorous filetype checking
                    let ext = path.extension().and_then(OsStr::to_str);
                    if !matches!(ext, Some("json")) {
                        files.push(f);
                    }
                }
            }
        }
        *self.files.write().await = files;
        self.has_changed.store(true, Ordering::Relaxed);
    }

    pub async fn list_content(&self, path: PathBuf) -> Result<Vec<String>, InstallError> {
        Ok(tokio::task::spawn_blocking(move || {
            let mut file = std::fs::File::open(path)?;
            list_archive_files(&mut file)
        })
        .await
        .unwrap()?)
    }

    pub async fn extract(
        &self,
        selected_index: usize,
        dest_dir_name: String,
        overwrite: bool,
    ) -> Result<(), InstallError> {
        let src_path = self.files.read().await.get(selected_index).unwrap().path();
        let mut dest_path = self.config.install_dir();
        dest_path.push(dest_dir_name);
        let mut jobs = self.extract_jobs.write().unwrap();

        if jobs.contains(&dest_path) {
            return Err(InstallError::InProgress);
        }

        if !overwrite && dest_path.exists() {
            return Err(InstallError::AlreadyExists);
        }

        jobs.insert(dest_path.clone());

        let logger = self.logger.clone();
        let extract_jobs = self.extract_jobs.clone();
        let has_changed = self.has_changed.clone();
        let _handle = std::thread::spawn(move || {
            match std::fs::File::open(&src_path) {
                Ok(src_file) => {
                    logger.log(format!("Begin extracting: {:?}", src_path.file_name().unwrap()));
                    match uncompress_archive(src_file, &dest_path, Ownership::Ignore) {
                        Ok(()) => {
                            logger.log(format!("Finished extracting: {:?}", src_path.file_name().unwrap()));
                        }
                        Err(e) => {
                            logger.log(format!("Extract failed with error: {:?}", e));
                        }
                    }
                }
                Err(e) => {
                    logger.log(format!("Unable to extract: {src_path:?} {:?}", e));
                }
            }
            let mut jobs = extract_jobs.write().unwrap();
            jobs.swap_remove(&dest_path);
            has_changed.store(true, Ordering::Relaxed);
        });
        Ok(())
    }
}
