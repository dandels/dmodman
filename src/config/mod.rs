pub mod config_error;
pub mod paths;

pub use config_error::ConfigError;
pub use paths::PathType;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConfigBuilder {
    pub apikey: Option<String>,
    pub game: Option<String>,
    pub download_dir: Option<String>,
}

impl ConfigBuilder {
    // used by unit test
    #[allow(dead_code)]
    pub fn default() -> Self {
        Self {
            apikey: None,
            game: None,
            download_dir: None,
        }
    }

    pub fn load() -> Result<Self, ConfigError> {
        let mut contents = String::new();
        let mut f = File::open(config_file())?;
        f.read_to_string(&mut contents)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn apikey<S: Into<String>>(mut self, apikey: S) -> Self {
        self.apikey = Some(apikey.into());
        self
    }

    pub fn game<S: Into<String>>(mut self, game: S) -> Self {
        self.game = Some(game.into());
        self
    }

    pub fn build(self) -> Result<Config, ConfigError> {
        if self.game.is_none() {
            return Err(ConfigError::GameMissing);
        }
        Ok(Config::new(self))
    }
}

#[derive(Clone)]
pub struct Config {
    pub apikey: Option<String>,
    pub game: String,
    pub download_dir: String,
}

impl Config {
    fn new(config: ConfigBuilder) -> Self {
        let download_dir = match config.download_dir {
            Some(dl_dir) => dl_dir,
            None => {
                if cfg!(test) {
                    format!(
                        "{}/test/downloads/{}",
                        env!("CARGO_MANIFEST_DIR"),
                        env!("CARGO_CRATE_NAME")
                    )
                } else {
                    format!(
                        "{}/{}",
                        dirs::download_dir().unwrap().to_string_lossy(),
                        env!("CARGO_CRATE_NAME")
                    )
                }
            }
        };

        Self {
            apikey: config.apikey,
            game: config.game.unwrap(),
            download_dir,
        }
    }

    pub fn cache_dir(&self) -> PathBuf {
        let mut path;
        if cfg!(test) {
            path = PathBuf::from(format!("{}/test/data", env!("CARGO_MANIFEST_DIR")));
        } else {
            path = dirs::data_local_dir().unwrap();
        }
        path.push(env!("CARGO_CRATE_NAME"));
        path
    }

    pub fn game_cache_dir(&self) -> PathBuf {
        let mut path = self.cache_dir();
        path.push(&self.game);
        path
    }

    pub fn download_dir(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.download_dir);
        path.push(&self.game);
        path
    }
}

pub fn config_file() -> PathBuf {
    let mut path;

    if cfg!(test) {
        path = PathBuf::from(format!("{}/test/config", env!("CARGO_MANIFEST_DIR")));
    } else {
        path = dirs::config_dir().unwrap();
    }

    path.push(env!("CARGO_CRATE_NAME"));
    path.push("config.toml");
    path
}

#[cfg(test)]
mod tests {
    use crate::config;
    use crate::config::{ConfigBuilder, ConfigError};

    #[test]
    fn read_apikey() -> Result<(), ConfigError> {
        let config = ConfigBuilder::load().unwrap();
        println!("{:?}", config::config_file());
        println!("{:?}", config.apikey);
        assert_eq!(config.apikey, Some("1234".to_string()));
        Ok(())
    }

    #[test]
    fn modfile_exists() -> Result<(), ConfigError> {
        let game = "morrowind";
        let modfile = "Graphic Herbalism MWSE - OpenMW-46599-1-03-1556986083.7z";
        let config = ConfigBuilder::default().game(game).build()?;
        let mut path = config.download_dir();
        path.push(modfile);
        println!("path: {:?}", path);
        assert!(path.exists());
        Ok(())
    }
}
