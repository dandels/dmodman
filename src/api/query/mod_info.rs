use super::Queriable;
use serde::{Deserialize, Serialize};
use crate::cache::Cacheable;

// TODO several of these should probably be Options
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModInfo {
    pub name: Option<String>,
    pub summary: Option<String>,
    #[serde(skip)]
    pub description: Option<String>,
    #[serde(skip)]
    pub picture_url: Option<String>,
    pub mod_id: u32,
    pub game_id: u32,
    pub domain_name: String,
    pub category_id: u32,
    pub version: Option<String>,
    pub created_timestamp: u64,
    pub created_time: String,
    pub updated_timestamp: u64,
    pub updated_time: String,
    pub author: String,
    pub uploaded_by: String,
    #[serde(skip)]
    pub uploaded_users_profile_url: Option<String>,
    pub contains_adult_content: bool,
    pub status: String,
    pub available: bool,
    #[serde(skip)]
    pub user: Option<UserInfo>,
    #[serde(skip)]
    pub endorsement: Option<Endorsement>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Endorsement {
    pub endorse_status: String,
    pub timestamp: Option<u32>,
    pub version: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub member_group_id: u32,
    pub member_id: u32,
    pub name: String,
}

impl Cacheable for ModInfo {}

impl Queriable for ModInfo {
    const FORMAT_STRING: &'static str = "games/{}/mods/{}.json";
}
