use super::Queriable;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DownloadLink {
    pub locations: Vec<Location>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub short_name: String,
    pub URI: String,
}

#[async_trait]
impl Queriable for DownloadLink {
    const FORMAT_STRING: &'static str = "games/{}/mods/{}/files/{}/download_link.json?{}";
}

#[cfg(test)]
mod tests {
    use super::DownloadLink;

    use crate::cache::Cacheable;
    use crate::config::ConfigBuilder;
    use crate::config::PathType;
    use std::error::Error;

    #[tokio::test]
    async fn deserialize_link_array() -> Result<(), Box<dyn Error>> {
        let game = "skyrimspecialedition";
        let config = ConfigBuilder::default().game(game).build().unwrap();
        let mod_id: u32 = 74484;
        let file_id: u64 = 1662417060;
        let path = config.path_for(PathType::DownloadLink(&game, &mod_id, &file_id));
        let links = DownloadLink::load(path).await.unwrap();
        assert_eq!(links.locations.get(1).unwrap().short_name, "Amsterdam");

        Ok(())
    }

    #[tokio::test]
    async fn deserialize_single_link() -> Result<(), Box<dyn Error>> {
        let game = "dragonage";
        let config = ConfigBuilder::default().game(game).build().unwrap();
        let mod_id: u32 = 343;
        let file_id: u64 = 5801;
        let path = config.path_for(PathType::DownloadLink(&game, &mod_id, &file_id));
        let links = DownloadLink::load(path).await.unwrap();
        assert_eq!(links.locations.first().unwrap().short_name, "Nexus CDN");

        Ok(())
    }
}
