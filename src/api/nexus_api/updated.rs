use crate::api::Queriable;
use crate::cache::Cacheable;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct Updated {
    pub updates: Vec<ModUpdate>,
}

#[derive(Deserialize, Serialize)]
pub struct ModUpdate {
    pub mod_id: u32,
    pub latest_file_update: u64,
    pub latest_mod_activity: u64,
}

impl Cacheable for Updated {}

impl Queriable for Updated {
    // fetch a list of all mods for a game updated within the past month
    const FORMAT_STRING: &'static str = "games/{}/mods/updated.json?period=1m";
}
