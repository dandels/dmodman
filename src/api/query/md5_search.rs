use super::{ModInfo, Queriable};
use crate::config;
use crate::db::Cacheable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Md5Search {
    pub results: Md5Results,
}

#[derive(Serialize, Deserialize)]
pub struct Md5Results {
    pub r#mod: ModInfo, // Needs to be named "mod" for serialization to succeed
    pub file_details: Md5FileDetails,
}

/* This is mostly the same as FileDetails, but it doesn't have a description field or size_kb field.
 * FileDetails on the other hand lacks the md5 sum.
 */
#[derive(Serialize, Deserialize)]
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
    pub uploaded_time: String,
    pub mod_version: Option<String>,
    pub external_virus_scan_url: Option<String>,
    pub changelog_html: Option<String>,
    pub md5: String,
}

impl Cacheable for Md5Search {
    const CACHE_DIR_NAME: &'static str = config::CACHE_DIR_MD5_SEARCH;
}

impl Queriable for Md5Search {
    const FORMAT_STRING: &'static str = "games/{}/mods/md5_search/{}.json";
}
