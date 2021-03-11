use std::error::Error;
use std::fmt;
use tokio::io;

#[derive(Debug)]
pub enum CacheError {
    IOError { source: io::Error },
    DeserializationError { source: serde_json::Error },
}

impl Error for CacheError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CacheError::IOError { ref source } => Some(source),
            CacheError::DeserializationError { ref source } => Some(source),
        }
    }
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            CacheError::IOError { source } => source.fmt(f),
            CacheError::DeserializationError { source } => source.fmt(f),
        }
    }
}

impl From<io::Error> for CacheError {
    fn from(error: io::Error) -> Self {
        CacheError::IOError { source: error }
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(error: serde_json::Error) -> Self {
        CacheError::DeserializationError { source: error }
    }
}
