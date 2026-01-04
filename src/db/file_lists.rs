use super::{CacheError, Cacheable};
use crate::api::{FileDetails, FileList};
use crate::config::{Config, DataPath};
use crate::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type Map<K, V> = Arc<RwLock<HashMap<K, V>>>;

#[derive(Clone)]
pub struct FileLists {
    map: Map<(String, u32), Option<Arc<FileList>>>,
    config: Config,
}

impl FileLists {
    pub fn new(config: Config) -> Result<Self, CacheError> {
        std::fs::create_dir_all(config.data_dir())?;

        Ok(Self {
            map: Default::default(),
            config,
        })
    }

    pub async fn insert<S: Into<String>>(&self, (game, mod_id): (S, u32), value: Arc<FileList>) {
        self.map.write().await.insert((game.into(), mod_id), Some(value));
    }

    pub async fn get<S: Into<String> + std::fmt::Display>(&self, game: S, mod_id: u32) -> Option<Arc<FileList>> {
        let game = game.into();
        let mut lock = self.map.write().await;
        match lock.get(&(game.clone(), mod_id)).cloned() {
            Some(fl) => fl,
            None => {
                let path: std::path::PathBuf = DataPath::FileList(&self.config, &game, mod_id).into();
                let fl_res = FileList::load(path.clone()).await;
                match fl_res {
                    Ok(fl) => {
                        let fl = Arc::new(fl);
                        lock.insert((game, mod_id), Some(fl.clone()));
                        Some(fl)
                    }
                    Err(e) => {
                        // BACKWARDS COMPATIBILITY
                        // Try once more with pre-v0.3.0 path
                        match FileList::load(DataPath::FileListCompat(&self.config, &game, mod_id)).await {
                            Ok(fl) => {
                                let fl = Arc::new(fl);
                                lock.insert((game, mod_id), Some(fl.clone()));
                                Some(fl)
                            }
                            Err(_) => {
                                log(format!("Failed to read file list from {path:?}:"));
                                log(format!("    {e}"));
                                // Cache negative result to reduce IO
                                lock.insert((game, mod_id), None);
                                None
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn filedetails_for(&self, game: String, mod_id: u32, file_id: u64) -> Option<Arc<FileDetails>> {
        let list = self.get(game, mod_id).await?;
        if let Ok(index) = list.files.binary_search_by(|fd| fd.file_id.cmp(&file_id)) {
            list.files.get(index).cloned()
        } else {
            None
        }
    }
}
