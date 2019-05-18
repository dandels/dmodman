use super::config;
use super::utils;
use crate::api::response::{DownloadLink, FileList, Md5SearchResults, ModInfo, NxmUrl};
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::path::PathBuf;

fn dl_link_path(nxm: &NxmUrl) -> PathBuf {
    let mut path = PathBuf::from(config::dl_links());
    path.push(&nxm.domain_name);
    path.push(&nxm.mod_id.to_string());
    utils::mkdir_recursive(&path);
    path.push(nxm.file_id.to_string() + ".json");
    path
}

pub fn save_dl_link(nxm: &NxmUrl, dl: &DownloadLink) -> Result<(), Error> {
    let path = dl_link_path(nxm);
    let mut file = File::create(&path)?;
    let data = serde_json::to_string_pretty(dl)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn read_dl_link(nxm: &NxmUrl) -> Result<DownloadLink, Error> {
    let path = dl_link_path(&nxm);
    let mut contents = String::new();
    let _n = File::open(path)?.read_to_string(&mut contents);
    let dl: DownloadLink = serde_json::from_str(&contents)?;
    Ok(dl)
}

fn mod_info_path(game: &str, mod_id: &u32) -> PathBuf {
    let mut path = config::mod_info();
    path.push(game);
    utils::mkdir_recursive(&path);
    path.push(mod_id.to_string() + ".json");
    path
}

pub fn save_mod_info(mi: &ModInfo) -> Result<(), std::io::Error> {
    let path = mod_info_path(&mi.domain_name, &mi.mod_id);
    let data = serde_json::to_string_pretty(mi)?;
    let mut file = File::create(&path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn read_mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, Error> {
    let path = mod_info_path(&game, &mod_id);
    let mut contents = String::new();;
    let _n = File::open(path)?.read_to_string(&mut contents)?;
    let mi: ModInfo =
        serde_json::from_str(&contents).expect("Unable to parse mod info file in cache");
    Ok(mi)
}

fn file_list_path(game: &str, mod_id: &u32) -> PathBuf {
    let mut path = config::file_lists();
    path.push(game);
    utils::mkdir_recursive(&path);
    path.push(mod_id.to_string() + ".json");
    path
}

pub fn save_file_list(game: &str, mod_id: &u32, fl: &FileList) -> Result<(), std::io::Error> {
    let path = file_list_path(&game, &mod_id);
    let mut file = File::create(&path)?;
    let data = serde_json::to_string_pretty(fl)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn read_file_list(game: &str, mod_id: &u32) -> Result<FileList, Error> {
    let path = file_list_path(&game, &mod_id);
    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;
    let fl: FileList = serde_json::from_str(&contents).expect("Unable to parse file list in cache");
    Ok(fl)
}

fn md5search_path(game: &str, md5: &str) -> PathBuf {
    let mut path = config::md5search();
    path.push(&game);
    utils::mkdir_recursive(&path);
    path.push(md5.to_string() + ".json");
    path
}

pub fn save_md5search(
    game: &str,
    md5: &str,
    results: &Md5SearchResults,
) -> Result<(), std::io::Error> {
    let path = md5search_path(game, &md5);
    let mut file = File::create(&path)?;
    let data = serde_json::to_string_pretty(&results)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn read_md5search(game: &str, md5: &str) -> Result<Md5SearchResults, Error> {
    let path = md5search_path(&game, &md5);
    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;
    let results: Md5SearchResults =
        serde_json::from_str(&contents).expect("Unable to parse file list in cache");
    Ok(results)
}
