use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use tokio::fs;
use tokio::fs::{DirEntry, File};

use crate::config::Config;
use crate::messages::Messages;

pub struct Archives {
    config: Config,
    msgs: Messages,
    has_changed: bool,
    files: Vec<DirEntry>,
}

enum ArchiveType {
    Lzma,
    Zip,
    Rar,
}

impl Archives {
    pub fn new(config: Config, msgs: Messages) -> Self {
        Self {
            config,
            msgs,
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

    pub async fn extract(&self, file: &PathBuf) {
        let atype: ArchiveType;
        if let Some(ext) = file.extension() {
            match ext.to_ascii_lowercase().to_str() {
                Some("7z") | Some("omod") => atype = ArchiveType::Lzma,
                Some("rar") => atype = ArchiveType::Rar,
                Some("zip") => atype = ArchiveType::Zip,
                None => return,
                Some(_) => {
                    self.msgs.push(format!("")).await;
                    return;
                }
            }
        } else {
            return;
        }

        if let ArchiveType::Lzma = atype {
            let _ = self.extract_7z(file).await;
        }
    }

    async fn extract_7z(&self, file: &PathBuf) -> Result<(), ()> {
        let f = File::open(file).await.unwrap();
        Ok(())
    }
}