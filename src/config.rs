use super::api::error::RequestError;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/* This approach for naming directories violates platform conventions on Windows and MacOS.
 * The "..\$Organization\$Project\" approach of Windows or the "org.$Organization.$Project/"
 * doesn't seem appropriate for an open source project.
 * Figure out a satisfying solution if we ever decide to support those platforms.
 */

pub const DOWNLOAD_DIR: &str = "downloads";
pub const CACHE_DIR_DL_LINKS: &str = "download_links";
pub const CACHE_DIR_FILE_DETAILS: &str = "file_lists";
pub const CACHE_DIR_FILE_LISTS: &str = "file_lists";
pub const CACHE_DIR_MOD_INFO: &str = "mod_info";
pub const CACHE_DIR_MD5_SEARCH: &str = "md5_search";

pub fn read_api_key() -> Result<String, RequestError> {
    let mut path: PathBuf = config_dir();
    path.push("apikey");
    let mut contents = String::new();
    match File::open(path) {
        Ok(mut f) => {
            f.read_to_string(&mut contents)?;
            Ok(contents.trim().to_string())
        }
        Err(_e) => Err(RequestError::ApiKeyMissing),
    }
}

pub fn game() -> Result<String, std::io::Error> {
    let mut path = config_dir();
    path.push("game");
    let mut contents = String::new();
    let _n = File::open(path)?.read_to_string(&mut contents);
    Ok(contents.trim().to_string())
}

pub fn cache_dir(game: &str) -> PathBuf {
    let mut path: PathBuf = dirs::data_local_dir().unwrap();
    path.push(clap::crate_name!());
    path.push(&game);
    path
}

fn config_dir() -> PathBuf {
    let mut path: PathBuf = dirs::config_dir().unwrap();
    path.push(clap::crate_name!());
    path
}

pub fn download_dir(game: &str) -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap();
    path.push(clap::crate_name!());
    path.push(&game);
    path.push(DOWNLOAD_DIR);
    path
}

#[cfg(test)]
mod tests {
    use crate::api::error::RequestError;
    use crate::config;
    use crate::test;

    #[test]
    fn apikey_exists() -> Result<(), RequestError> {
        test::setup();
        config::read_api_key()?;
        Ok(())
    }
}
