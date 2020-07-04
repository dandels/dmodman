use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadLink {
    pub location: Location,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub short_name: String,
    pub uri: String,
}
