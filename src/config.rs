use super::file;
use std::path::PathBuf;

pub fn get_data_dir() -> PathBuf {
    let mut data_dir: PathBuf = dirs::data_local_dir().unwrap();
    data_dir.push("dmodman");
    file::create_dir_if_not_exist(&data_dir);
    data_dir
}

pub fn get_config_dir() -> PathBuf {
    let mut config_dir: PathBuf = dirs::config_dir().unwrap();
    config_dir.push("dmodman");
    file::create_dir_if_not_exist(&config_dir);
    config_dir
}

pub fn get_download_list_dir() -> PathBuf {
    let mut cache_dir = get_data_dir();
    cache_dir.push("mod_download_list");
    file::create_dir_if_not_exist(&cache_dir);
    cache_dir
}

pub fn get_cache_dir() -> PathBuf {
    let mut cache_dir = get_data_dir();
    cache_dir.push("mod_info_cache");
    file::create_dir_if_not_exist(&cache_dir);
    cache_dir
}

// TODO don't read this for every request
pub fn get_api_key() -> String {
    let mut apikey: PathBuf = get_config_dir();
    apikey.push("apikey");
    let s: &str = apikey.to_str().unwrap();
    let errmsg: &str = &format!("No API key found in {}", s);
    return file::file_to_string(&apikey).expect(errmsg);
}

fn get_game_config_path() -> String {
    let mut file = get_config_dir();
    file.push("game");
    file.to_str().unwrap().to_string()
}

// TODO implement actual settings
pub fn get_game() -> Result<String, std::io::Error> {
    let mut file = get_config_dir();
    file.push("game");
    return file::file_to_string(&file)
}

pub fn set_game(game: &str) -> Result<(), std::io::Error> {
    return file::write_file(&PathBuf::from(get_game_config_path()), game)
}
