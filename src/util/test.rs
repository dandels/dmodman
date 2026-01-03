use std::sync::Arc;

use crate::{
    config::{Config, ConfigBuilder},
    events::{EventTx, Events},
    logger::Logger,
};

pub fn init_structs() -> (Arc<Config>, Logger, EventTx) {
    let event_tx = Events::new().tx;
    let logger = Logger::new(event_tx.clone(), false);
    let cfg = Arc::new(ConfigBuilder::from_defaults(logger.clone()).build().unwrap());
    (cfg, logger, event_tx)
}

pub fn init_structs_with_profile(profile: &str) -> (Arc<Config>, Logger, EventTx) {
    let event_tx = Events::new().tx;
    let logger = Logger::new(event_tx.clone(), false);
    let cfg = Arc::new(ConfigBuilder::from_defaults(logger.clone()).profile(profile).build().unwrap());
    (cfg, logger, event_tx)
}
