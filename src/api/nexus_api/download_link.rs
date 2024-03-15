use crate::api::Queriable;
use crate::cache::Cacheable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DownloadLink {
    pub locations: Vec<Location>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Location {
    pub name: String,
    pub short_name: String,
    pub URI: String,
}

impl Cacheable for DownloadLink {}

impl Queriable for DownloadLink {
    const FORMAT_STRING: &'static str = "games/{}/mods/{}/files/{}/download_link.json?{}";
}

#[cfg(test)]
mod tests {
    use super::DownloadLink;

    use crate::cache::Cacheable;
    use crate::config::ConfigBuilder;
    use crate::config::DataPath;
    use std::error::Error;

    #[tokio::test]
    async fn deserialize_link_array() -> Result<(), Box<dyn Error>> {
        let profile = "testprofile";
        let game = "skyrimspecialedition";
        let config = ConfigBuilder::default().profile(profile).build().unwrap();
        let mod_id: u32 = 74484;
        let file_id: u64 = 1662417060;
        let path = DataPath::DownloadLink(&config, game, mod_id, file_id);
        let links = DownloadLink::load(path).await.unwrap();
        assert_eq!(links.locations.get(1).unwrap().short_name, "Amsterdam");

        Ok(())
    }

    #[tokio::test]
    async fn deserialize_single_link() -> Result<(), Box<dyn Error>> {
        let profile = "testprofile";
        let game = "dragonage";
        let config = ConfigBuilder::default().profile(profile).build().unwrap();
        let mod_id: u32 = 343;
        let file_id: u64 = 5801;
        let path = DataPath::DownloadLink(&config, game, mod_id, file_id);
        let links = DownloadLink::load(path).await.unwrap();
        assert_eq!(links.locations.first().unwrap().short_name, "Nexus CDN");

        Ok(())
    }
}
