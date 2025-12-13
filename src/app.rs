use crate::Config;
use crate::Logger;
use crate::api::Client;
use crate::api::Downloads;
use crate::api::Query;
use crate::db::Db;
use crate::events::EventTx;
use std::sync::Arc;

// This is just a wrapper type to simplify passing all of these around
// Naming things is hard
pub struct App {
    pub ctrl: AppCtrl,
    pub client: Client,
    pub db: Db,
    pub query: Query,
    pub downloads: Downloads,
}

impl App {
    pub fn new(ctrl: AppCtrl) -> Self {
        let client = Client::new(ctrl.clone());
        let db = Db::new(ctrl.clone());
        let query = Query::new(ctrl.clone(), db.clone(), client.clone());
        let downloads = Downloads::new(ctrl.clone(), db.clone(), client.clone(), query.clone());
    }
}

// Another wrapper for the things that need to be passed almost everywhere
#[derive(Clone)]
pub struct AppCtrl {
    pub config: Arc<Config>,
    pub event_tx: EventTx,
    pub logger: Logger,
}

impl AppCtrl {
    pub fn from_config(config: Arc<Config>) -> Self {
        let events = EventTx::default();

        Self {
            config,
            logger: Logger::new(events.clone(), false),
            event_tx: events,
        }
    }

    #[cfg(test)]
    pub fn test_default() -> Self {
        Self::test_with_config(Config::default().into())
    }

    #[cfg(test)]
    pub fn test_with_config(config: Arc<Config>) -> Self {
        let (events, _) = crate::events::create_channel();
        Self {
            config,
            logger: Logger::new(events.clone(), false),
            event_tx: events,
        }
    }
}
