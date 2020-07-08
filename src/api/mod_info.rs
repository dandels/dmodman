use serde::{Deserialize, Serialize};

// TODO several of these should probably be Options
#[derive(Debug, Serialize, Deserialize)]
pub struct ModInfo {
    pub name: String,
    pub summary: String,
    pub description: String,
    pub picture_url: Option<String>,
    pub mod_id: u32,
    pub game_id: u32,
    pub domain_name: String,
    pub category_id: u32,
    pub version: String,
    pub created_timestamp: u64,
    pub created_time: String,
    pub updated_timestamp: u64,
    pub updated_time: String,
    pub author: String,
    pub uploaded_by: String,
    pub uploaded_users_profile_url: String,
    pub contains_adult_content: bool,
    pub status: String,
    pub available: bool,
    pub user: Option<UserInfo>,
    pub endorsement: Option<Endorsement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Endorsement {
    pub endorse_status: String,
    pub timestamp: Option<u32>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub member_group_id: u32,
    pub member_id: u32,
    pub name: String,
}
