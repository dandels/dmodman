use super::config;
use super::file;
use crate::api::{DownloadLocation, FileList, ModInfo, NxmUrl};
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::path::PathBuf;

pub fn save_dl_loc(nxm: &NxmUrl, dl: &DownloadLocation) -> Result<(), Error> {
    let mut path = PathBuf::from(config::dl_cache_dir());
    path.push(&nxm.domain_name);
    path.push(&nxm.mod_id.to_string());
    file::create_dir_if_not_exist(&path);
    path.push(nxm.file_id.to_string() + ".json");
    let mut file = File::create(&path)?;
    let data = serde_json::to_string_pretty(dl)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn read_dl_loc(nxm: &NxmUrl) -> Result<DownloadLocation, Error> {
    let path = config::dl_loc_for_file(&nxm.domain_name, &nxm.mod_id, &nxm.file_id);
    let contents = file::read_to_string(&path)?;
    let dl: DownloadLocation = serde_json::from_str(&contents).unwrap();
    Ok(dl)
}

pub fn save_mod_info(mi: &ModInfo) -> Result<(), std::io::Error> {
    let path = config::mod_info_path(&mi.domain_name, &mi.mod_id);
    let mut file = File::create(&path)?;
    let data = serde_json::to_string_pretty(mi)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn read_mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, Error> {
    let path = config::mod_info_path(game, mod_id);
    let contents = file::read_to_string(&path)?;
    let mi: ModInfo =
        serde_json::from_str(&contents).expect("Unable to parse mod info file in cache");
    Ok(mi)
}

pub fn read_file_list(game: &str, mod_id: &u32) -> Result<FileList, Error> {
    let path = config::file_list_path(&game, &mod_id);
    let contents = file::read_to_string(&path)?;
    let fl: FileList = serde_json::from_str(&contents).expect("Unable to parse file list in cache");
    Ok(fl)
}
pub fn save_file_list(game: &str, mod_id: &u32, fl: &FileList) -> Result<(), std::io::Error> {
    let mut path = PathBuf::from(config::file_list_dir());
    path.push(game);
    file::create_dir_if_not_exist(&path);
    path.push(mod_id.to_string() + ".json");
    let mut file = File::create(&path)?;
    let data = serde_json::to_string_pretty(fl)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}
