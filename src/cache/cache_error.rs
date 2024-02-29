use std::error::Error;
use std::fmt;
use tokio::io;

#[derive(Debug)]
pub enum CacheError {
    IO { source: io::Error },
    Deserialization { source: serde_json::Error },
}

impl Error for CacheError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CacheError::IO { ref source } => Some(source),
            CacheError::Deserialization { ref source } => Some(source),
        }
    }
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CacheError::IO { source } => source.fmt(f),
            CacheError::Deserialization { source } => source.fmt(f),
        }
    }
}

impl From<io::Error> for CacheError {
    fn from(error: io::Error) -> Self {
        CacheError::IO { source: error }
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(error: serde_json::Error) -> Self {
        CacheError::Deserialization { source: error }
    }
}
