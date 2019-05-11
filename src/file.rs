use super::config;
use crate::api::{DownloadList, ModInfo};
use std::fs::File;
use std::fs::Metadata;
use std::io::prelude::*;
use std::path::PathBuf;

pub fn save_mod_info(mi: &ModInfo) -> Result<(), std::io::Error> {
    let mut path = PathBuf::from(config::get_cache_dir());
    path.push(&mi.domain_name);
    create_dir_if_not_exist(&path);
    path.push(mi.mod_id.to_string() + ".json");
    let mut file = File::create(&path)?;
    let data = serde_json::to_string_pretty(mi)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn save_download_list(
    game: &str,
    mod_id: &u32,
    dl: &DownloadList,
) -> Result<(), std::io::Error> {
    let mut path = PathBuf::from(config::get_download_list_dir());
    path.push(game);
    create_dir_if_not_exist(&path);
    path.push(mod_id.to_string() + ".json");
    let mut file = File::create(&path)?;
    let data = serde_json::to_string_pretty(dl)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn read_download_list(game: &str, mod_id: &u32) -> Option<DownloadList> {
    let mut path = config::get_download_list_dir();
    path.push(game);
    path.push(mod_id.to_string() + ".json");
    let opt_contents = file_to_string(&path);
    match opt_contents {
        Ok(v) => {
            println!("Found download list in cache");
            let dl: DownloadList = serde_json::from_str(&v).unwrap();
            return Some(dl)
        }
        Err(_) => {
            println!("Unable to find download info info in cache.");
            return None;
        }
    }
}

pub fn read_mod_info(game: &str, mod_id: &u32) -> Option<ModInfo> {
    let mut path = config::get_cache_dir();
    path.push(game);
    path.push(mod_id.to_string() + ".json");
    let opt_contents = file_to_string(&path);
    match opt_contents {
        Ok(v) => {
            println!("Found mod info in cache.");
            let mi: ModInfo = serde_json::from_str(&v).unwrap();
            return Some(mi)
        }
        Err(_) => {
            println!("Unable to find mod info in cache.");
            return None;
        }
    }
}

pub fn write_file(path: &PathBuf, data: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(&path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn file_to_string(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut r = File::open(path)?;
    let mut contents: String = String::new();
    r.read_to_string(&mut contents)?;
    Ok(contents.trim().to_string())
}

pub fn create_dir_if_not_exist(path: &PathBuf) {
    let opt_md = path.metadata();
    let md: Metadata;
    match opt_md {
        Ok(v) => md = v.to_owned(),
        Err(_v) => {
            std::fs::create_dir(path.to_str().unwrap()).expect(&format!(
                "Unable to create directory at {}",
                path.to_str().unwrap()
            ));
            md = path.metadata().unwrap();
        }
    }
    assert!(md.is_dir());
}
