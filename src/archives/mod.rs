use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use compress_tools::tokio_support::*;
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

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub async fn list_contents(&self, path: &PathBuf) {
        let file = File::open(path).unwrap();
        let list = compress_tools::list_archive_files(&file).unwrap();
        for l in list {
            self.msgs.push(l).await;
        }
    }
}
