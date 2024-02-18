pub mod config_error;
pub mod paths;

pub use config_error::ConfigError;
pub use paths::PathType;

use crate::util;

use std::env;
use std::io::prelude::Write;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, fs::File};

use serde::Deserialize;

/* The ConfigBuilder is loaded based on the config file, or initialized with empty values. It's used for deserializing
 * and setting config values that might be missing. We then turn it into a proper Config, which let's us avoid wrapping
 * most settings inside an Option. */
#[derive(Default, Deserialize)]
pub struct ConfigBuilder {
    // API key can be stored in either config or separate file (when generated for user). Config takes precedence.
    pub apikey: Option<String>,
    pub profile: Option<String>,
    pub download_dir: Option<String>,
    pub install_dir: Option<String>,
}

#[allow(dead_code)]
impl ConfigBuilder {
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

    pub fn profile<S: Into<String>>(mut self, profile: S) -> Self {
        self.profile = Some(profile.into());
        self
    }

    pub fn download_dir<S: Into<String>>(mut self, dir: S) -> Self {
        self.download_dir = Some(dir.into());
        self
    }

    pub fn install_dir<S: Into<String>>(mut self, dir: S) -> Self {
        self.install_dir = Some(dir.into());
        self
    }

    pub fn build(mut self) -> Result<Config, ConfigError> {
        if self.apikey.is_none() {
            self.apikey = try_read_apikey().ok();
        }

        Ok(Config::new(self)?)
    }
}

#[derive(Clone)]
pub struct Config {
    pub apikey: Option<String>,
    pub profile: Option<String>,
    download_dir: String,
    install_dir: String,
}

impl Config {
    fn new(config: ConfigBuilder) -> Result<Self, ConfigError> {
        let download_dir = match config.download_dir {
            Some(dl_dir) => shellexpand::full(&dl_dir)?.to_string(),
            None => {
                if cfg!(test) {
                    format!("{}/test/downloads/{}", env!("CARGO_MANIFEST_DIR"), env!("CARGO_CRATE_NAME"))
                } else {
                    format!("{}/{}", dirs::download_dir().unwrap().to_string_lossy(), env!("CARGO_CRATE_NAME"))
                }
            }
        };

        let install_dir = match config.install_dir {
            Some(ins_dir) => shellexpand::full(&ins_dir)?.to_string().into(),
            None => {
                if cfg!(test) {
                    format!("{}/test/data/{}", env!("CARGO_MANIFEST_DIR"), env!("CARGO_CRATE_NAME")).into()
                } else {
                    format!("{}/{}", dirs::data_dir().unwrap().to_string_lossy(), env!("CARGO_CRATE_NAME")).into()
                }
            }
        };

        Ok(Self {
            apikey: config.apikey,
            profile: config.profile,
            download_dir,
            install_dir,
        })
    }

    pub fn cache_dir(&self) -> PathBuf {
        let mut path = dirs::cache_dir().unwrap();
        path.push(env!("CARGO_CRATE_NAME"));
        path
    }

    pub fn data_dir(&self) -> PathBuf {
        let mut path;
        if cfg!(test) {
            path = PathBuf::from(format!("{}/test/data", env!("CARGO_MANIFEST_DIR")));
        } else {
            path = dirs::data_local_dir().unwrap();
        }
        path.push(env!("CARGO_CRATE_NAME"));
        path
    }

    pub fn download_dir(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.download_dir);
        if let Some(profile) = &self.profile {
            path.push(profile);
        }
        path
    }

    pub fn install_dir(&self) -> PathBuf {
        let mut path = PathBuf::from(&self.install_dir);
        match &self.profile {
            Some(profile) => path.push(profile),
            // Maybe generate an error instead if this is unconfigured
            None => path.push("default"),
        }
        path
    }

    pub fn save_apikey(&self) -> Result<(), std::io::Error> {
        fs::create_dir_all(config_dir())?;
        let mut f = File::create(apikey_file())?;
        f.write_all(self.apikey.as_ref().unwrap().as_bytes())?;
        f.flush()
    }
}

pub fn config_dir() -> PathBuf {
    let mut path;

    if cfg!(test) {
        path = PathBuf::from(format!("{}/test/config", env!("CARGO_MANIFEST_DIR")));
    } else {
        path = dirs::config_dir().unwrap();
    }
    path.push(env!("CARGO_CRATE_NAME"));
    path
}

pub fn config_file() -> PathBuf {
    let mut path = config_dir();
    path.push("config.toml");
    path
}

pub fn apikey_file() -> PathBuf {
    let mut path = config_dir();
    path.push("apikey");
    path
}

pub fn try_read_apikey() -> Result<String, std::io::Error> {
    let mut contents = String::new();
    let mut f = File::open(apikey_file())?;
    f.read_to_string(&mut contents)?;
    Ok(util::trim_newline(contents))
}

#[cfg(test)]
mod tests {
    use crate::config::{ConfigBuilder, ConfigError};
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn read_apikey() -> Result<(), ConfigError> {
        let config = ConfigBuilder::load().unwrap().build()?;
        assert_eq!(config.apikey, Some("1234".to_string()));
        Ok(())
    }

    #[test]
    fn modfile_exists() -> Result<(), ConfigError> {
        let profile = "morrowind";
        let modfile = "Graphic Herbalism MWSE - OpenMW-46599-1-03-1556986083.7z";
        let config = ConfigBuilder::default().profile(profile).build()?;
        let mut path = config.download_dir();
        path.push(modfile);
        println!("path: {:?}", path);
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn expand_env_variable() -> Result<(), ConfigError> {
        env::set_var("MY_VAR", "/opt/games/dmodman");
        let config = ConfigBuilder::default().download_dir("$MY_VAR").profile("skyrim").build()?;
        assert_eq!(PathBuf::from("/opt/games/dmodman/skyrim"), config.download_dir());
        Ok(())
    }

    #[test]
    fn expand_tilde() -> Result<(), ConfigError> {
        env::set_var("HOME", "/home/dmodman_test");
        let config = ConfigBuilder::default().download_dir("~/downloads").profile("stardew valley").build()?;
        assert_eq!(PathBuf::from("/home/dmodman_test/downloads/stardew valley"), config.download_dir());
        Ok(())
    }

    #[test]
    fn expand_complex_path() -> Result<(), ConfigError> {
        env::set_var("HOME", "/root/subdir");
        env::set_var("FOO_VAR", "foo/bar");
        let config =
            ConfigBuilder::default().download_dir("~/secret$FOO_VAR").profile("?!\"¤%😀 my profile").build()?;
        assert_eq!(PathBuf::from("/root/subdir/secretfoo/bar/?!\"¤%😀 my profile"), config.download_dir());
        Ok(())
    }
}
