use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Games {
    games: Vec<GameInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct GameInfo {
    id: u64,
    name: String,
    forum_url: String,
    nexusmods_url: String,
    genre: String,
    file_count: u64,
    downloads: u64,
    domain_name: String,
    approved_date: u64,
    file_views: u64,
    authors: u64,
    file_endorsements: u64,
    mods: u64,
    categories: Vec<Category>,
}

#[derive(Serialize, Deserialize)]
struct Category {
    category_id: u64,
    name: String,
    parent_category: bool,
}
