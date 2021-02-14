use super::config;
use crate::api::*;
use std::fs::File;
use std::io::{Error, Read};
use std::path::PathBuf;

fn dl_link_path(nxm: &NxmUrl) -> Result<PathBuf, Error> {
    let mut path = PathBuf::from(config::CACHE_DIR_DL_LINKS);
    path.push(&nxm.domain_name);
    path.push(nxm.mod_id.to_string());
    path.push(nxm.file_id.to_string() + ".json");
    Ok(path)
}

pub fn read_dl_link(nxm: &NxmUrl) -> Result<DownloadLink, Error> {
    let path = dl_link_path(&nxm)?;
    let mut contents = String::new();
    let _n = File::open(path)?.read_to_string(&mut contents);
    let dl: DownloadLink = serde_json::from_str(&contents)?;
    Ok(dl)
}

pub fn read_mod_info(game: &str, mod_id: &u32) -> Result<ModInfo, Error> {
    let mut path = PathBuf::from(config::CACHE_DIR_MOD_INFO);
    path.push(game);
    path.push(mod_id.to_string() + ".json");
    let mut contents = String::new();
    let _n = File::open(path)?.read_to_string(&mut contents)?;
    let mi: ModInfo =
        serde_json::from_str(&contents).expect("Unable to parse mod info file in cache");
    Ok(mi)
}
