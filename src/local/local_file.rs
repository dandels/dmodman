use crate::api::NxmUrl;
use crate::config;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Error, Write};

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalFile {
    pub file_name: String,
    pub game: String,
    pub mod_id: u32,
    pub file_id: u64,
}

impl LocalFile {
    pub fn new(nxm: &NxmUrl, path: &std::path::Path) -> Self {
        LocalFile {
            file_name: path.file_name().unwrap().to_str().unwrap().to_owned(),
            game: nxm.domain_name.to_owned(),
            mod_id: nxm.mod_id,
            file_id: nxm.file_id,
        }
    }

    pub fn write(&self) -> Result<(), Error> {
        let mut path = config::download_dir(&self.game);
        path.push(&self.file_name);
        let mut name: String = path.to_str().unwrap().to_owned();
        name.push_str(".json");

        println!("Creating metadata file for {:?}", name);
        let mut file: File = File::create(name)?;

        let data = serde_json::to_string_pretty(&self)?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }
}
