use std::error::Error;
use std::fmt;
use tokio::io;

#[derive(Debug)]
pub enum ConfigError {
    IO {
        source: io::Error,
    },
    Deserialization {
        source: toml::de::Error,
    },
    ShellExpand {
        source: shellexpand::LookupError<std::env::VarError>,
    },
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::IO { ref source } => Some(source),
            ConfigError::Deserialization { ref source } => Some(source),
            ConfigError::ShellExpand { ref source } => Some(source),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::IO { source } => source.fmt(f),
            ConfigError::Deserialization { source } => source.fmt(f),
            ConfigError::ShellExpand { source } => source.fmt(f),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        ConfigError::IO { source: error }
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(error: toml::de::Error) -> Self {
        ConfigError::Deserialization { source: error }
    }
}

impl From<shellexpand::LookupError<std::env::VarError>> for ConfigError {
    fn from(error: shellexpand::LookupError<std::env::VarError>) -> Self {
        ConfigError::ShellExpand { source: error }
    }
}
