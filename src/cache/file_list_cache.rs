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
    map: Arc<RwLock<HashMap<(String, u32), FileList>>>,
}

impl FileListCache {
    pub async fn new(config: &Config) -> Result<Self, CacheError> {
        let mut file_lists: HashMap<(String, u32), FileList> = HashMap::new();

        let mut fl_dir: PathBuf = config.cache_dir();
        fl_dir.push(paths::FILE_LISTS);

        // Iterates over the entries in cache_dir/file_lists/<game>/<mod_id>.json and deserializes them into FileLists
        let mut stream = fs::read_dir(&fl_dir).await?;
        while let Some(subdir) = stream.next_entry().await? {
            let game_name = subdir.file_name();
            let mut inner_stream = fs::read_dir(subdir.path()).await?;
            while let Some(f) = inner_stream.next_entry().await? {
                if f.path().is_file() && f.path().extension().and_then(OsStr::to_str) == Some("json") {
                    if let Some(filename) = f.path().file_stem() {
                        if let Ok(mod_id) = str::parse::<u32>(&filename.to_string_lossy()) {
                            if let Ok(fl) = FileList::load(f.path()).await {
                                file_lists.insert((game_name.to_string_lossy().into_owned(), mod_id), fl);
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

    pub async fn insert<S: Into<String>>(&self, (game, mod_id): (S, u32), value: FileList) {
        self.map.write().await.insert((game.into(), mod_id), value);
    }

    pub async fn get(&self, (game, mod_id): (&str, u32)) -> Option<FileList> {
        match self.map.read().await.get(&(game.to_string(), mod_id)) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
}
