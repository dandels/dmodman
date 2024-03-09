use super::{CacheError, Cacheable};
use crate::api::{FileDetails, FileList};
use crate::config::{Config, DataType};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct FileLists {
    #[allow(clippy::type_complexity)]
    map: Arc<RwLock<HashMap<(String, u32), Option<FileList>>>>,
    config: Config,
}

impl FileLists {
    // TODO read file lists from disk only on-demand, so we don't pointlessly deserialize data for other games
    pub async fn new(config: Config) -> Result<Self, CacheError> {
        fs::create_dir_all(config.data_dir()).await?;

        Ok(Self {
            map: Default::default(),
            config,
        })
    }

    pub async fn insert<S: Into<String>>(&self, (game, mod_id): (S, u32), value: FileList) {
        self.map.write().await.insert((game.into(), mod_id), Some(value));
    }

    /* TODO could the FileLists and FileDetails be wrapped in Arcs? Then the FileDetails wouldn't be cloned for every
     * file */
    pub async fn get<S: Into<String> + std::fmt::Display>(&self, game: S, mod_id: u32) -> Option<FileList> {
        let game = game.into();
        let mut lock = self.map.write().await;
        match lock.get(&(game.clone(), mod_id)).cloned() {
            Some(fl) => fl,
            None => match FileList::load(self.config.path_for(DataType::FileList(&game, mod_id))).await {
                Ok(fl) => {
                    lock.insert((game, mod_id), Some(fl.clone()));
                    Some(fl)
                }
                Err(_) => {
                    // Cache negative result to reduce IO
                    lock.insert((game, mod_id), None);
                    None
                }
            },
        }
    }

    pub async fn filedetails_for(&self, game: String, mod_id: u32, file_id: u64) -> Option<Arc<FileDetails>> {
        self.get(game, mod_id).await.and_then(|list| {
            list.files.get(list.files.binary_search_by(|fd| fd.file_id.cmp(&file_id)).unwrap()).cloned()
        })
    }
}
