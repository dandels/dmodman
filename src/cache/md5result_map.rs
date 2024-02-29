use super::Cacheable;
use crate::api::Md5Results;
use crate::config::DataType;
use crate::{Config, Logger};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::RwLock;

type Map<K, V> = Arc<RwLock<HashMap<K, V>>>;

#[derive(Clone)]
pub struct Md5ResultMap {
    config: Config,
    #[allow(dead_code)]
    logger: Logger,
    map: Map<(String, u64), Option<Md5Results>>,
}

impl Md5ResultMap {
    pub fn new(config: Config, logger: Logger) -> Self {
        Self {
            config,
            logger,
            map: Default::default(),
        }
    }

    pub async fn insert(&self, game: String, res: Md5Results) {
        self.map.write().await.insert((game, res.file_details.file_id), Some(res));
    }

    pub async fn get<S: Into<String> + Display>(&self, game: S, file_id: u64) -> Option<Md5Results> {
        let game = game.into();
        let mut lock = self.map.write().await;
        match lock.get(&(game.clone(), file_id)).cloned() {
            Some(fl) => fl,
            None => match Md5Results::load(self.config.path_for(DataType::Md5Results(&game, file_id))).await {
                Ok(res) => {
                    lock.insert((game, file_id), Some(res.clone()));
                    Some(res)
                }
                Err(_) => {
                    // Cache negative result
                    lock.insert((game, file_id), None);
                    None
                }
            },
        }
    }
}
