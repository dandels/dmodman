use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

// TODO implement actual settings for all these things

// TODO don't read this for every request?
pub fn api_key() -> Result<String, std::io::Error> {
    let mut path: PathBuf = config_dir();
    path.push("apikey");
    let mut contents = String::new();
    let _n = File::open(path)?.read_to_string(&mut contents);
    let ret = contents.trim();
    Ok(ret.to_string())
}

pub fn game() -> Result<String, std::io::Error> {
    let mut path = config_dir();
    path.push("game");
    let mut contents = String::new();
    let _n = File::open(path)?.read_to_string(&mut contents);
    let trimmed = contents.trim();
    Ok(trimmed.to_string())
}

fn data_dir() -> PathBuf {
    let mut data_dir: PathBuf = dirs::data_local_dir().expect("Unable to find cache dir location.");
    data_dir.push(clap::crate_name!());
    data_dir
}

pub fn log_dir() -> PathBuf {
    return data_dir();
}

fn config_dir() -> PathBuf {
    let mut path: PathBuf = dirs::config_dir().expect("Unable to find config dir location.");
    path.push(clap::crate_name!());
    path
}

pub fn downloads() -> PathBuf {
    let mut path = data_dir();
    path.push("downloads");
    path
}

pub fn download_location_for(game: &str, mod_id: &u32) -> PathBuf {
    let mut path = downloads();
    path.push(&game);
    path.push(&mod_id.to_string());
    path
}

pub fn dl_links() -> PathBuf {
    let mut path = data_dir();
    path.push("download_links");
    path
}
pub fn file_lists() -> PathBuf {
    let mut path = data_dir();
    path.push("file_lists");
    path
}

pub fn mod_info() -> PathBuf {
    let mut path = data_dir();
    path.push("mod_info");
    path
}

pub fn md5_search() -> PathBuf {
    let mut path = data_dir();
    path.push("md5_search");
    path
}
