use super::api::error::DownloadError;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/* This approach for naming directories violates platform conventions on Windows and MacOS.
 * The "..\$Organization\$Project\" approach of Windows or the "org.$Organization.$Project/"
 * doesn't seem appropriate for an open source project.
 * TODO: figure out a satisfying solution.
 */

pub const DIR_DOWNLOADS: &str = "downloads";
pub const CACHE_DIR_DL_LINKS: &str = "download_links";
pub const CACHE_DIR_FILE_DETAILS: &str = "file_lists";
pub const CACHE_DIR_FILE_LISTS: &str = "file_lists";
pub const CACHE_DIR_MOD_INFO: &str = "mod_info";
pub const CACHE_DIR_MD5_SEARCH: &str = "md5_search";

// Not yet stabilized
/*
pub const CACHE_DIR: PathBuf = dirs::cache_dir().unwrap();
pub const CONFIG_DIR: PathBuf = dirs::config_dir().unwrap();
pub const DATA_DIR: PathBuf = dirs::data_local_dir().unwrap();
// TODO this needs to be configurable
pub const DOWNLOAD_DIR: PathBuf = dirs::data_local_dir().unwrap();
pub const LOG_DIR: PathBuf = DATA_DIR;
*/

pub fn read_api_key() -> Result<String, DownloadError> {
    let mut path: PathBuf = config_dir();
    path.push("apikey");
    let mut contents = String::new();
    match File::open(path) {
        Ok(mut f) => {
            f.read_to_string(&mut contents)?;
            Ok(contents.trim().to_string())
        }
        Err(_e) => Err(DownloadError::ApiKeyMissing),
    }
}

pub fn game() -> Result<String, std::io::Error> {
    let mut path = config_dir();
    path.push("game");
    let mut contents = String::new();
    let _n = File::open(path)?.read_to_string(&mut contents);
    Ok(contents.trim().to_string())
}

fn config_dir() -> PathBuf {
    let mut path: PathBuf = dirs::config_dir().expect("Unable to find config dir location.");
    path.push(clap::crate_name!());
    path
}

fn config_file() -> PathBuf {
    let mut path = config_dir();
    path.push("config");
    path
}

pub fn download_location_for(game: &str, mod_id: &u32) -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap();
    path.push(&game);
    path.push(&mod_id.to_string());
    path
}
