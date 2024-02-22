use std::path::PathBuf;

use compress_tools::*;
use std::ffi::OsStr;
// This module mixes std and tokio fs, be mindful which one we're using
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::fs;
use tokio::fs::DirEntry;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::logger::Logger;

#[derive(Clone)]
pub struct Archives {
    config: Config,
    logger: Logger,
    pub has_changed: Arc<AtomicBool>, // used by UI to ask if error list needs to be redrawn
    pub files: Arc<RwLock<Vec<DirEntry>>>,
}

impl Archives {
    pub fn new(config: Config, logger: Logger) -> Self {
        Self {
            config,
            logger,
            has_changed: AtomicBool::new(true).into(),
            files: Arc::new(vec![].into()),
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

    pub async fn list_contents(&self, path: PathBuf) -> Result<Vec<String>> {
        tokio::task::spawn_blocking(move || {
            let mut file = File::open(path).unwrap();
            list_archive_files(&mut file)
        })
        .await?
    }

    pub async fn extract(&self, selected_index: usize, dest_dir_name: String) {
        let src_path = self.files.read().await.get(selected_index).unwrap().path();
        let mut dest_path = self.config.install_dir();

        let logger = self.logger.clone();
        std::thread::spawn(move || match File::open(&src_path) {
            Ok(src_file) => {
                dest_path.push(dest_dir_name);
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
        });
    }
}
