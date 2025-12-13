use super::Cacheable;
use crate::api::ModInfo;
use crate::config::DataPath;
use crate::{Config, Logger};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::RwLock;

type Map<K, V> = Arc<RwLock<HashMap<K, V>>>;

#[derive(Clone)]
pub struct ModInfoMap {
    config: Arc<Config>,
    #[allow(dead_code)]
    logger: Logger,
    map: Map<(String, u32), Option<Arc<ModInfo>>>,
}

impl ModInfoMap {
    pub fn new(config: Arc<Config>, logger: Logger) -> Self {
        Self {
            config,
            logger,
            map: Default::default(),
        }
    }

    pub async fn insert(&self, modinfo: Arc<ModInfo>) {
        self.map.write().await.insert((modinfo.domain_name.clone(), modinfo.mod_id), Some(modinfo));
    }

    pub async fn get<S: Into<String> + Display>(&self, game: S, mod_id: u32) -> Option<Arc<ModInfo>> {
        let game = game.into();
        let mut lock = self.map.write().await;
        match lock.get(&(game.clone(), mod_id)).cloned() {
            Some(fl) => fl,
            None => match ModInfo::load(DataPath::ModInfo(&self.config, &game, mod_id)).await {
                Ok(res) => {
                    let res = Arc::new(res);
                    lock.insert((game, mod_id), Some(res.clone()));
                    Some(res)
                }
                Err(_) => {
                    // Cache negative result
                    lock.insert((game, mod_id), None);
                    None
                }
            },
        }
    }
}


