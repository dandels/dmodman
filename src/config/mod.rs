pub mod config_error;
pub mod paths;

pub use config_error::ConfigError;
pub use paths::DataPath;

use super::Logger;
use crate::util;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::io::prelude::Write;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, fs::File};

/* The ConfigBuilder is loaded based on the config file, or initialized with empty values. It's used for deserializing
 * and setting config values that might be missing. We then turn it into a proper Config, which let's us avoid wrapping
 * most settings inside an Option.
 *
 * Download_dir and install_dir have default values but can be overriden per profile.
 * install_dir does not have a configurable global setting because appending $profile to it would be a nuisance to
 * the user, and extracting all mods to the same directory leads to a mess.
 *
 * The original behavior of download_dir is to append $profile to its path in case $profile is set.
 * This behavior is kept for backwards compatibility reasons in case profiles is None, or the active Profile does not
 * specify a download directory. */
#[derive(Default, Deserialize)]
pub struct ConfigBuilder {
    pub apikey: Option<String>,
    pub profile: Option<String>,
    #[serde(alias = "global_download_dir")]
    pub download_dir: Option<PathBuf>,
    #[serde(alias = "global_install_dir")]
    pub install_dir: Option<PathBuf>,
    pub profiles: HashMap<String, Profile>,
    #[serde(skip)]
    logger: Logger,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Profile {
    pub download_dir: Option<PathBuf>,
    pub install_dir: Option<PathBuf>,
}

const DEFAULT_PROFILE_NAME: &str = "default";

impl ConfigBuilder {
    pub fn load(logger: Logger) -> Result<Self, ConfigError> {
        let mut contents = String::new();

        let mut f = File::open(config_file())?;
        f.read_to_string(&mut contents)?;

        let mut loaded: ConfigBuilder = toml::from_str(&contents)?;
        loaded.apply_settings_from_profile();

        Ok(Self { logger, ..loaded })
    }

    fn apply_settings_from_profile(&mut self) {
        if let Some(selected_profile) = &self.profile {
            if let Some(profile) = self.profiles.get(selected_profile) {
                if let Some(dls_dir) = &profile.download_dir {
                    self.download_dir = Some(dls_dir.to_owned());
                };
                if let Some(ins_dir) = &profile.install_dir {
                    self.install_dir = Some(ins_dir.to_owned());
                };
            }
        }
    }

    #[allow(dead_code)]
    pub fn apikey<S: Into<String>>(mut self, apikey: S) -> Self {
        self.apikey = Some(apikey.into());
        self
    }

    #[allow(dead_code)]
    pub fn profile<S: Into<String>>(mut self, profile: S) -> Self {
        self.profile = Some(profile.into());
        self.apply_settings_from_profile();
        self
    }

    #[allow(dead_code)]
    pub fn download_dir<S: Into<String>>(mut self, dir: S) -> Self {
        self.download_dir = Some(PathBuf::from(dir.into()));
        self
    }

    #[allow(dead_code)]
    pub fn install_dir<S: Into<String>>(mut self, dir: S) -> Self {
        self.install_dir = Some(PathBuf::from(dir.into()));
        self
    }

    pub fn build(mut self) -> Result<Config, ConfigError> {
        // API key can be stored in the config or a separate file (default). Config takes precedence.
        if self.apikey.is_none() {
            self.apikey = try_read_apikey().ok();
        }

        // Fallback behavior for missing settings.
        match &self.profile {
            Some(selected_profile) => match self.profiles.get(selected_profile) {
                Some(profile) => {
                    if profile.download_dir.is_none() && self.download_dir.is_none() {
                        self.download_dir = Some(default_download_dir().join(selected_profile));
                    }
                    if profile.install_dir.is_none() && self.install_dir.is_none() {
                        self.install_dir = Some(install_dir_for_profile(selected_profile));
                    }
                }
                None => {
                    self.download_dir = match self.download_dir {
                        Some(dls) => Some(dls.join(selected_profile)),
                        None => Some(default_download_dir().join(selected_profile)),
                    };
                    self.install_dir = match self.install_dir {
                        Some(ins) => Some(ins.join(selected_profile)),
                        None => Some(install_dir_for_profile(selected_profile)),
                    }
                }
            },
            None => {
                if self.download_dir.is_none() {
                    self.download_dir = Some(default_download_dir());
                }
                if self.install_dir.is_none() {
                    self.install_dir = Some(install_dir_for_profile(DEFAULT_PROFILE_NAME));
                }
            }
        }

        self.download_dir = Some(shellexpand::full(&self.download_dir.unwrap().to_string_lossy())?.to_string().into());
        self.install_dir = Some(shellexpand::full(&self.install_dir.unwrap().to_string_lossy())?.to_string().into());

        Config::new(self.logger.clone(), self)
    }
}

// The dirs crate reads ~/.config/user-dirs.dirs directly and ignores environment variables. This messes up tests.
pub fn xdg_download_dir() -> PathBuf {
    match env::var("XDG_DOWNLOAD_DIR") {
        Ok(val) if val.starts_with("$HOME") || val.starts_with('/') => PathBuf::from(val),
        _ => dirs::download_dir().unwrap(),
    }
}

pub fn xdg_data_dir() -> PathBuf {
    match env::var("XDG_DATA_DIR") {
        Ok(val) if val.starts_with("$HOME") || val.starts_with('/') => PathBuf::from(val),
        _ => dirs::data_dir().unwrap(),
    }
}

pub fn default_download_dir() -> PathBuf {
    xdg_download_dir().join(env!("CARGO_CRATE_NAME"))
}

pub fn install_dir_for_profile(profile: &str) -> PathBuf {
    xdg_data_dir().join(env!("CARGO_CRATE_NAME")).join("profiles").join(profile).join("install")
}

#[derive(Clone)]
pub struct Config {
    pub apikey: Option<String>,
    profile: String,
    download_dir: PathBuf,
    install_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        ConfigBuilder::default().build().unwrap()
    }
}

impl Config {
    fn new(logger: Logger, config: ConfigBuilder) -> Result<Self, ConfigError> {
        let download_dir = {
            let path = config.download_dir.expect("Config was passed Builder with missing download dir.");
            match path.is_absolute() {
                true => path,
                false => {
                    logger.log("Download dir is not an absolute path. Using path relative to $HOME.");
                    let mut home = dirs::home_dir().unwrap();
                    home.push(path);
                    home
                }
            }
        };

        let install_dir = {
            let path = config.install_dir.expect("Config was passed Builder with missing install dir.");
            match path.is_absolute() {
                true => path,
                false => {
                    logger.log("Install dir is not an absolute path. Using path relative to $HOME.");
                    let mut home = dirs::home_dir().unwrap();
                    home.push(path);
                    home
                }
            }
        };

        Ok(Self {
            apikey: config.apikey,
            profile: config.profile.unwrap_or("default".to_string()),
            download_dir,
            install_dir,
        })
    }

    pub fn cache_for_profile(&self) -> PathBuf {
        let mut path = dirs::cache_dir().unwrap();
        path.push(env!("CARGO_CRATE_NAME"));
        path.push(&self.profile);
        path
    }

    pub fn metadata_for_profile(&self) -> PathBuf {
        let mut path = self.data_dir();
        path.push("profiles");
        path.push(&self.profile);
        path.push("metadata");
        path
    }

    pub fn metadata_dir(&self) -> PathBuf {
        self.data_dir().join("metadata")
    }

    pub fn data_dir(&self) -> PathBuf {
        let mut path;
        if cfg!(test) {
            path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/data");
        } else {
            path = dirs::data_local_dir().unwrap();
        }
        path.push(env!("CARGO_CRATE_NAME"));
        path
    }

    pub fn download_dir(&self) -> PathBuf {
        self.download_dir.clone()
    }

    pub fn install_dir(&self) -> PathBuf {
        self.install_dir.clone()
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
        path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/config");
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
pub mod tests {
    use crate::config::*;
    use std::env;

    pub fn setup_env() {
        unsafe {
            env::set_var("HOME", format!("{}/test/", env!("CARGO_MANIFEST_DIR")));
            env::set_var("XDG_DATA_DIR", "$HOME/data");
            env::set_var("XDG_DOWNLOAD_DIR", "$HOME/downloads");
        }
    }

    #[test]
    fn read_apikey() -> Result<(), ConfigError> {
        setup_env();
        let config = ConfigBuilder::load(Logger::default()).unwrap().build()?;
        assert_eq!(config.apikey, Some("1234".to_string()));
        Ok(())
    }

    #[test]
    fn modfile_exists() -> Result<(), ConfigError> {
        setup_env();
        let profile = "testprofile";
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
        unsafe { env::set_var("MY_VAR", "/opt/games/dmodman"); }
        let config = ConfigBuilder::default().download_dir("$MY_VAR").profile("skyrim").build()?;
        assert_eq!(PathBuf::from("/opt/games/dmodman/skyrim"), config.download_dir());
        Ok(())
    }

    #[test]
    fn expand_tilde() -> Result<(), ConfigError> {
        unsafe { env::set_var("HOME", "/home/dmodman_test"); }
        let config = ConfigBuilder::default().download_dir("~/downloads").profile("stardew valley").build()?;
        assert_eq!(PathBuf::from("/home/dmodman_test/downloads/stardew valley"), config.download_dir());
        Ok(())
    }

    #[test]
    fn expand_complex_path() -> Result<(), ConfigError> {
        unsafe {
            env::set_var("HOME", "/root/subdir");
            env::set_var("FOO_VAR", "foo/bar");
        }
        let config =
            ConfigBuilder::default().download_dir("~/secret$FOO_VAR").profile("?!\"¤%😀 my profile").build()?;
        assert_eq!(PathBuf::from("/root/subdir/secretfoo/bar/?!\"¤%😀 my profile"), config.download_dir());
        Ok(())
    }

    #[test]
    fn default_config() -> Result<(), ConfigError> {
        unsafe {
            env::set_var("HOME", "/home/dmodman_test");
            env::set_var("XDG_DATA_DIR", "$HOME/.local/share");
            env::set_var("XDG_DOWNLOAD_DIR", "$HOME/Downloads");
        }
        let config = ConfigBuilder::default().build()?;
        println!("dirs {:?}", dirs::download_dir());
        assert_eq!(PathBuf::from("/home/dmodman_test/Downloads/dmodman"), config.download_dir());
        assert_eq!(
            PathBuf::from("/home/dmodman_test/.local/share/dmodman/profiles/default/install"),
            config.install_dir()
        );
        Ok(())
    }

    #[test]
    fn append_profile_to_dirs() -> Result<(), ConfigError> {
        setup_env();
        unsafe { env::set_var("HOME", "/home/dmodman_test") };
        let config = ConfigBuilder::load(Logger::default())?.profile("append").build()?;
        assert_eq!(PathBuf::from("/home/dmodman_test/toplevel_dls/append"), config.download_dir());
        assert_eq!(PathBuf::from("/home/dmodman_test/toplevel_ins/append"), config.install_dir());
        Ok(())
    }

    #[test]
    fn relative_paths() -> Result<(), ConfigError> {
        unsafe { env::set_var("HOME", "/home/dmodman_test") };
        let config = ConfigBuilder::load(Logger::default())?.profile("relative_test").build()?;
        assert_eq!(PathBuf::from("/home/dmodman_test/relative_dls/"), config.download_dir());
        assert_eq!(PathBuf::from("/home/dmodman_test/relative_ins/"), config.install_dir());
        Ok(())
    }

    #[test]
    fn absolute_paths() -> Result<(), ConfigError> {
        unsafe { env::set_var("HOME", "/home/dmodman_test") };
        let config = ConfigBuilder::load(Logger::default())?.profile("absolute_test").build()?;
        assert_eq!(PathBuf::from("/absolute_dls"), config.download_dir());
        assert_eq!(PathBuf::from("/absolute_ins"), config.install_dir());
        Ok(())
    }

    #[test]
    fn profile_specific_install_dir() -> Result<(), ConfigError> {
        unsafe { env::set_var("HOME", "/home/dmodman_test") };
        let config = ConfigBuilder::load(Logger::default())?.profile("insdir_only_test").build()?;
        assert_eq!(PathBuf::from("/home/dmodman_test/insdir_only"), config.install_dir());
        assert_eq!(PathBuf::from("/home/dmodman_test/toplevel_dls"), config.download_dir());
        Ok(())
    }
}
