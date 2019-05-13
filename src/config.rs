use super::file;
use std::path::PathBuf;

pub fn dl_cache_dir() -> PathBuf {
    let mut cache_dir = data_dir();
    cache_dir.push("mod_dl_cache");
    cache_dir
}

pub fn data_dir() -> PathBuf {
    let mut data_dir: PathBuf = dirs::data_local_dir().unwrap();
    data_dir.push("dmodman");
    data_dir
}

pub fn config_dir() -> PathBuf {
    let mut config_dir: PathBuf = dirs::config_dir().unwrap();
    config_dir.push("dmodman");
    config_dir
}

pub fn file_list_dir() -> PathBuf {
    let mut cache_dir = data_dir();
    cache_dir.push("mod_file_list");
    cache_dir
}

pub fn cache_dir() -> PathBuf {
    let mut cache_dir = data_dir();
    cache_dir.push("mod_info_cache");
    cache_dir
}

// TODO don't read this for every request
pub fn api_key() -> Result<String, std::io::Error> {
    let mut apikey: PathBuf = config_dir();
    apikey.push("apikey");
    return file::read_to_string(&apikey);
}

// TODO implement actual settings
pub fn game() -> Result<String, std::io::Error> {
    let mut file = config_dir();
    file.push("game");
    return file::read_to_string(&file);
}

pub fn download_dir(game: &str) -> PathBuf {
    let mut data_dir: PathBuf = dirs::data_local_dir().unwrap();
    data_dir.push("dmodman");
    data_dir.push("downloads");
    data_dir.push(game);
    data_dir
}

pub fn dl_loc_for_file(game: &str, mod_id: &u32, file_id: &u64) -> PathBuf {
    let mut path = file_list_dir();
    path.push(game);
    path.push(mod_id.to_string());
    file::create_dir_if_not_exist(&path);
    path.push(file_id.to_string() + ".json");
    path
}

pub fn file_list_path(game: &str, mod_id: &u32) -> PathBuf {
    let mut path = file_list_dir();
    path.push(game);
    file::create_dir_if_not_exist(&path);
    path.push(mod_id.to_string() + ".json");
    path
}

pub fn mod_info_path(game: &str, mod_id: &u32) -> PathBuf {
    let mut path = cache_dir();
    path.push(&game);
    file::create_dir_if_not_exist(&path);
    path.push(mod_id.to_string() + ".json");
    path
}
