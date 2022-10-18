use crate::api::error::RequestError;
use crate::api::Client;
use crate::util::format;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadLinks {
    pub locations: Vec<Location>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub short_name: String,
    pub URI: String,
}

impl DownloadLinks {
    const FORMAT_STRING: &'static str = "games/{}/mods/{}/files/{}/download_link.json?{}";

    pub async fn request(client: &Client, params: Vec<&str>) -> Result<DownloadLinks, RequestError> {
        let endpoint = format::vec_with_format_string(Self::FORMAT_STRING, params);
        let resp = client.send_api_request(&endpoint).await?.error_for_status()?;
        let ret: Vec<Location> = serde_json::from_value(resp.json().await?).unwrap();
        Ok(DownloadLinks { locations: ret })
    }
}
