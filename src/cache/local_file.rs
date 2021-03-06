use super::error::CacheError;
use crate::api::NxmUrl;
use crate::config;
use serde::{Deserialize, Serialize};
use tokio::{fs, fs::File};
use tokio::io::{Error, AsyncWriteExt};
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocalFile {
    pub game: String,
    pub file_name: String,
    pub mod_id: u32,
    pub file_id: u64,
}

impl LocalFile {
    pub fn new(nxm: &NxmUrl, file_name: String) -> Self {
        LocalFile {
            game: nxm.domain_name.to_owned(),
            file_name,
            mod_id: nxm.mod_id,
            file_id: nxm.file_id,
        }
    }

    pub fn path(&self) -> PathBuf {
        let mut path = config::download_dir(&self.game);
        path.push(&self.file_name);
        path
    }

    pub async fn from_str(arg: &str) -> Result<Self, CacheError> {
        Ok(serde_json::from_str(&fs::read_to_string(&arg).await?)?)
    }

    pub async fn from_path(path: &Path) -> Result<Self, CacheError> {
        Ok(serde_json::from_str(&fs::read_to_string(&path).await?)?)
    }

    pub async fn write(&self) -> Result<(), Error> {
        let mut path = config::download_dir(&self.game);
        path.push(&self.file_name);
        let mut name: String = path.to_str().unwrap().to_owned();
        name.push_str(".json");

        let mut file: File = File::create(name).await?;

        let data = serde_json::to_string_pretty(&self)?;
        file.write_all(data.as_bytes()).await?;
        Ok(())
    }
}
