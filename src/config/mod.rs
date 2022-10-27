use super::api::error::RequestError;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

/* This approach for naming directories violates platform conventions on Windows and MacOS.
 * The "..\$Organization\$Project\" approach of Windows or the "org.$Organization.$Project/"
 * doesn't seem appropriate for an open source project.
 * Figure out a satisfying solution if we ever decide to support those platforms.
 */

pub const DOWNLOAD_DIR: &str = "downloads";

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
    let mut path;
    if cfg!(test) {
        path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
        path.push("test");
        path.push("data");
    } else {
        path = dirs::data_local_dir().unwrap();
    }
    path.push(clap::crate_name!());
    path.push(&game);
    path
}

fn config_dir() -> PathBuf {
    let mut path;
    if cfg!(test) {
        path = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
        path.push("test");
        path.push("config");
    } else {
        path = dirs::config_dir().unwrap();
    }
    path.push(clap::crate_name!());
    path
}

pub fn download_dir(game: &str) -> PathBuf {
    let mut path = cache_dir(game);
    path.push(DOWNLOAD_DIR);
    path
}

#[cfg(test)]
mod tests {
    use crate::api::error::RequestError;
    use crate::config;

    #[test]
    fn apikey_exists() -> Result<(), RequestError> {
        config::read_api_key()?;
        Ok(())
    }
}
