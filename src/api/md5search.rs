use super::ModInfo;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize)]
pub struct Md5SearchResults {
    /* The map contains:
     * mod: Md5Search
     * file_details: Md5FileDetails
     */
    pub results: Map<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Md5Search {
    pub mod_info: ModInfo,
    pub md5_file_details: Md5FileDetails,
}

/* This is mostly the same as FileDetails, but it doesn't have a description field or size field.
 * FileDetails on the other hand lacks the md5 sum.
 * We should try use composition here and hope that serde is able to deserialize it.
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

pub fn parse_results(results: &Map<String, Value>) -> Md5Search {
    let mijson = results["mod"].to_owned();
    let mi: ModInfo = serde_json::from_value(mijson).unwrap();
    let fdjson = results["file_details"].to_owned();
    let fd: Md5FileDetails = serde_json::from_value(fdjson).unwrap();
    return Md5Search {
        mod_info: mi,
        md5_file_details: fd,
    };
}
