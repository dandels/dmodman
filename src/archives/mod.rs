use std::ffi::OsStr;
use std::path::PathBuf;

use compress_tools::*;
// This module mixes std and tokio fs, be mindful which one we're using
use std::fs::File;
use tokio::fs;
use tokio::fs::DirEntry;

use crate::config::Config;
use crate::logger::Logger;

pub struct Archives {
    config: Config,
    logger: Logger,
    has_changed: bool,
    pub files: Vec<DirEntry>,
}

impl Archives {
    pub fn new(config: Config, logger: Logger) -> Self {
        Self {
            config,
            logger,
            has_changed: true,
            files: vec![],
        }
    }

    pub fn swap_has_changed(&mut self) -> bool {
        let ret = self.has_changed;
        self.has_changed = false;
        ret
    }

    pub async fn list(&mut self) -> &Vec<DirEntry> {
        let mut ret: Vec<DirEntry> = vec![];
        if let Ok(mut dir_entries) = fs::read_dir(self.config.download_dir()).await {
            // TODO log errors since this shouldn't fail
            while let Ok(Some(f)) = dir_entries.next_entry().await {
                if f.path().is_file() {
                    let path = f.path();
                    let ext = path.extension().and_then(OsStr::to_str);
                    // TODO case sensitivity
                    if matches!(ext, Some("7z") | Some("zip") | Some("rar")) {
                        ret.push(f);
                    }
                }
            }
        }
        self.files = ret;
        &self.files
    }

    pub async fn list_contents(&self, path: PathBuf) -> Result<Vec<String>> {
        tokio::task::spawn_blocking(move || {
            let mut file = File::open(path).unwrap();
            list_archive_files(&mut file)
        })
        .await?
    }

    pub async fn extract(&self, selected_index: usize, dest_dir_name: String) {
        let src_path = self.files.get(selected_index).unwrap().path();
        let mut dest_path = self.config.download_dir();

        let logger = self.logger.clone();
        tokio::task::spawn_blocking(move || match File::open(&src_path) {
            Ok(src_file) => {
                dest_path.push(dest_dir_name);
                if let Err(e) = uncompress_archive(src_file, &dest_path, Ownership::Ignore) {
                    logger.log(format!("Extract failed with error: {:?}", e));
                }
            }
            Err(e) => {
                logger.log(format!("Unable to extract: {src_path:?} {:?}", e));
            }
        });
    }
}
