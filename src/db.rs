use super::config;
use super::file;
use crate::api::{FileList, ModInfo};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub fn save_mod_info(mi: &ModInfo) -> Result<(), std::io::Error> {
    let mut path = PathBuf::from(config::get_cache_dir());
    path.push(&mi.domain_name);
    file::create_dir_if_not_exist(&path);
    path.push(mi.mod_id.to_string() + ".json");
    let mut file = File::create(&path)?;
    let data = serde_json::to_string_pretty(mi)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn save_file_list(game: &str, mod_id: &u32, fl: &FileList) -> Result<(), std::io::Error> {
    let mut path = PathBuf::from(config::get_file_list_dir());
    path.push(game);
    file::create_dir_if_not_exist(&path);
    path.push(mod_id.to_string() + ".json");
    let mut file = File::create(&path)?;
    let data = serde_json::to_string_pretty(fl)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn read_file_list(game: &str, mod_id: &u32) -> Option<FileList> {
    let mut path = config::get_file_list_dir();
    path.push(game);
    path.push(mod_id.to_string() + ".json");
    let opt_contents = file::read_to_string(&path);
    match opt_contents {
        Ok(v) => {
            println!("Found file list in cache");
            let fl: FileList = serde_json::from_str(&v).unwrap();
            return Some(fl);
        }
        Err(_) => {
            println!("Unable to find download info in cache.");
            return None;
        }
    }
}

pub fn read_mod_info(game: &str, mod_id: &u32) -> Option<ModInfo> {
    let mut path = config::get_cache_dir();
    path.push(game);
    path.push(mod_id.to_string() + ".json");
    let opt_contents = file::read_to_string(&path);
    match opt_contents {
        Ok(v) => {
            println!("Found mod info in cache.");
            let mi: ModInfo = serde_json::from_str(&v).unwrap();
            return Some(mi);
        }
        Err(_) => {
            println!("Unable to find mod info in cache.");
            return None;
        }
    }
}
