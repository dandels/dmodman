use super::ModInfo;
use crate::api::Queriable;
use crate::cache::Cacheable;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Md5Search {
    pub results: Vec<Md5Result>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Md5Result {
    #[serde(alias = "mod")] // field is called "mod" in the response
    pub mod_info: Arc<ModInfo>,
    pub file_details: Arc<Md5FileDetails>,
}

/* This is mostly the same as FileDetails, but it doesn't have an id, description or size_kb field.
 * FileDetails on the other hand lacks the md5 sum.
 */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Md5FileDetails {
    pub file_id: u64,
    pub name: String,
    pub version: Option<String>,
    pub category_id: u32,
    pub category_name: Option<String>,
    pub is_primary: bool,
    pub size: u64,
    pub file_name: String,
    pub uploaded_timestamp: u64,
    #[serde(skip)]
    pub uploaded_time: String,
    pub mod_version: Option<String>,
    #[serde(skip)]
    pub external_virus_scan_url: Option<String>,
    #[serde(skip)]
    pub changelog_html: Option<String>,
    pub md5: String,
}

impl Cacheable for Md5Result {}
impl Queriable for Md5Search {
    const FORMAT_STRING: &'static str = "games/{}/mods/md5_search/{}.json";
}
