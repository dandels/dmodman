use super::Cacheable;
use crate::config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadLink {
    pub location: Location,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub short_name: String,
    pub URI: String,
}

impl Cacheable for DownloadLink {
    const CACHE_DIR_NAME: &'static str = config::CACHE_DIR_DL_LINKS;
}
