use std::error::Error;
use std::fmt;
use tokio::io;

#[derive(Debug)]
pub enum ConfigError {
    IOError { source: io::Error },
    DeserializationError { source: toml::de::Error },
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::IOError { ref source } => Some(source),
            ConfigError::DeserializationError { ref source } => Some(source),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::IOError { source } => source.fmt(f),
            ConfigError::DeserializationError { source } => source.fmt(f),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        ConfigError::IOError { source: error }
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(error: toml::de::Error) -> Self {
        ConfigError::DeserializationError { source: error }
    }
}
