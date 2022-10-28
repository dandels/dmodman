use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

use serde::Deserialize;
use super::ConfigError;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub apikey: Option<String>,
    pub cross_game_modding: Option<bool>,
    pub game: Option<String>,
    download_dir: Option<String>,
}

impl Config {
    pub fn new(game_arg: Option<&str>, nxm_game_opt: Option<String>) -> Result<Self, ConfigError> {
        let mut contents = String::new();
        let mut f = File::open(config_file())?;
        f.read_to_string(&mut contents)?;
        let mut config: Self = toml::from_str(&contents)?;

        if let Some(game) = game_arg {
            config.game = Some(game.to_string())
        } else if let Some(true) = config.cross_game_modding {
            if let Some(nxm_game) = nxm_game_opt {
                config.game = Some(nxm_game)
            }
        }

        Ok(config)
    }

    pub fn game_cache_dir(&self) -> PathBuf {
        let mut path;
        if cfg!(test) {
            path = PathBuf::from(format!("{}/test/data", env!("CARGO_MANIFEST_DIR")));
        } else {
            path = dirs::data_local_dir().unwrap();
        }
        path.push(clap::crate_name!());
        path.push(self.game.clone().unwrap());
        path
    }

    pub fn download_dir(&self) -> PathBuf {
        let mut path;
        match &self.download_dir {
            Some(dl_dir) => path = PathBuf::from_str(&dl_dir).unwrap(),
            None => {
                if cfg!(test) {
                    path = PathBuf::from(format!("{}/test/downloads", env!("CARGO_MANIFEST_DIR")));
                } else {
                    path = dirs::download_dir().unwrap();
                }
                path.push(clap::crate_name!());
            }
        }
        path.push(self.game.clone().unwrap());
        path
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
        assert_eq!(config.apikey, Some("1234".to_string()));
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
