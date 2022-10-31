use super::{CacheError, Cacheable};
use crate::api::FileList;
use crate::config::{paths, Config};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct FileListCache {
    map: Arc<RwLock<HashMap<u32, FileList>>>,
}

impl FileListCache {
    pub async fn new(config: &Config) -> Result<Self, CacheError> {
        let mut file_lists: HashMap<u32, FileList> = HashMap::new();

        let mut fl_dir: PathBuf = config.game_cache_dir();
        fl_dir.push(paths::FILE_LISTS);

        if let Ok(mut file_stream) = fs::read_dir(fl_dir).await {
            while let Some(f) = file_stream.next_entry().await? {
                if f.path().is_file() && f.path().extension().and_then(OsStr::to_str) == Some("json") {
                    if let Some(filename) = f.path().file_stem() {
                        if let Ok(mod_id) = str::parse::<u32>(&filename.to_string_lossy()) {
                            if let Ok(fl) = FileList::load(f.path()).await {
                                file_lists.insert(mod_id, fl);
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            map: Arc::new(RwLock::new(file_lists)),
        })
    }

    pub async fn insert(&self, key: u32, value: FileList) {
        self.map.write().await.insert(key, value);
    }

    pub async fn get(&self, key: &u32) -> Option<FileList> {
        match self.map.read().await.get(key) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
}
