use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{atomic::AtomicBool, Arc, RwLock};

use super::ConfigError;
use serde::Deserialize;

#[derive(Deserialize)]
struct ParsedConfig {
    apikey: Option<String>,
    cross_game_modding: Option<bool>,
    game: Option<String>,
    download_dir: Option<String>,
}

#[derive(Clone)]
pub struct Config {
    apikey: Arc<RwLock<Option<String>>>,
    pub cross_game_modding: Arc<AtomicBool>,
    game: Arc<RwLock<Option<String>>>,
    download_dir: Arc<RwLock<String>>,
}

impl Config {
    pub fn new(game_arg: Option<&str>, nxm_game_opt: Option<String>) -> Result<Self, ConfigError> {
        let mut contents = String::new();
        let mut f = File::open(config_file())?;
        f.read_to_string(&mut contents)?;
        let mut config: ParsedConfig = toml::from_str(&contents)?;

        if let Some(game) = game_arg {
            println!("REACHABLE CODE");
            config.game = Some(game.to_string())
        } else if let Some(true) = config.cross_game_modding {
            if let Some(nxm_game) = nxm_game_opt {
                config.game = Some(nxm_game)
            }
        }

        let cross_game_modding = match config.cross_game_modding {
            Some(true) => AtomicBool::new(true),
            _ => AtomicBool::new(false),
        };

        let download_dir = match config.download_dir {
            Some(dl_dir) => dl_dir,
            None => {
                if cfg!(test) {
                    format!("{}/test/downloads/{}", env!("CARGO_MANIFEST_DIR"), clap::crate_name!())
                } else {
                    format!(
                        "{:?}/{}",
                        dirs::download_dir().unwrap().to_string_lossy(),
                        clap::crate_name!()
                    )
                }
            }
        };

        Ok(Self {
            apikey: Arc::new(RwLock::new(config.apikey)),
            game: Arc::new(RwLock::new(config.game)),
            cross_game_modding: Arc::new(cross_game_modding),
            download_dir: Arc::new(RwLock::new(download_dir)),
        })
    }

    pub fn game_cache_dir(&self) -> PathBuf {
        let mut path;
        if cfg!(test) {
            path = PathBuf::from(format!("{}/test/data", env!("CARGO_MANIFEST_DIR")));
        } else {
            path = dirs::data_local_dir().unwrap();
        }
        path.push(clap::crate_name!());
        path.push(self.game().unwrap());
        path
    }

    pub fn download_dir(&self) -> PathBuf {
        let mut path = PathBuf::from_str(&(*self.download_dir.read().unwrap())).unwrap();
        path.push(self.game().unwrap());
        path
    }

    pub fn apikey(&self) -> Option<String> {
        return self.apikey.read().unwrap().clone()
    }

    pub fn game(&self) -> Option<String> {
        return self.game.read().unwrap().clone()
    }
}

fn config_file() -> PathBuf {
    let mut path;

    if cfg!(test) {
        path = PathBuf::from(format!("{}/test/config", env!("CARGO_MANIFEST_DIR")));
    } else {
        path = dirs::config_dir().unwrap();
    }

    path.push(clap::crate_name!());
    path.push("config.toml");
    path
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::config::ConfigError;

    #[test]
    fn read_apikey() -> Result<(), ConfigError> {
        let config = Config::new(None, None).unwrap();
        assert_eq!(*config.apikey, Some("1234".to_string()));
        Ok(())
    }

    #[test]
    fn modfile_exists() -> Result<(), ConfigError> {
        let game = "morrowind";
        let modfile = "Graphic Herbalism MWSE - OpenMW-46599-1-03-1556986083.7z";
        let config = Config::new(Some(game), None).unwrap();
        let mut path = config.download_dir();
        path.push(modfile);
        println!("path: {:?}", path);
        assert!(path.exists());
        Ok(())
    }
}
