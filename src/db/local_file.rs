use super::error::DbError;
use crate::api::{Cacheable, FileDetails, FileList, NxmUrl};
use crate::config;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::path::PathBuf;

pub struct LocalFileList {
    game: String,
    pub files: Vec<LocalFile>,
    pub file_details: Vec<FileDetails>,
}

impl LocalFileList {
    pub fn new(game: &str) -> Result<Self, DbError> {
        Ok(Self::with_file_details(&game, vec![])?)
    }

    // Creates LocalFileList from files in download directory that end with .json
    pub fn with_file_details(game: &str, file_details: Vec<FileDetails>) -> Result<Self, DbError> {
        let files = fs::read_dir(config::download_dir(&game))?
            .flatten()
            .filter_map(|x| {
                if x.path().is_file()
                    && x.path().extension().and_then(OsStr::to_str) == Some("json")
                {
                    Some(LocalFile::from_path(&x.path()).unwrap())
                } else {
                    None
                }
            })
            .collect();

        Ok(Self {
            game: game.to_owned(),
            files,
            file_details,
        })
    }

    pub fn populate_file_details(&mut self) -> Vec<FileDetails> {
        self.files
            .iter()
            .filter_map(|x| {
                if let Ok(fl) = FileList::try_from_cache(&self.game, &x.mod_id) {
                    Some(fl.files.iter().find(|fd| fd.file_id == x.file_id))
                } else {
                    None
                }
            })
            .flatten()
            .cloned()
            .collect()
    }
}

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

    fn from_str(arg: &str) -> Result<Self, DbError> {
        Ok(serde_json::from_str(&std::fs::read_to_string(&arg)?)?)
    }

    fn from_path(path: &Path) -> Result<Self, DbError> {
        Ok(serde_json::from_str(&std::fs::read_to_string(&path)?)?)
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
