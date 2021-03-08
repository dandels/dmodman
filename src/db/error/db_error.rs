use std::error::Error;
use std::fmt;
use tokio::io;

#[derive(Debug)]
pub enum DbError {
    IOError { source: io::Error },
    SerializationError { source: serde_json::Error },
}

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DbError::IOError { ref source } => Some(source),
            DbError::SerializationError { ref source } => Some(source),
        }
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            DbError::IOError { source } => source.fmt(f),
            DbError::SerializationError { source } => source.fmt(f),
        }
    }
}

impl From<io::Error> for DbError {
    fn from(error: io::Error) -> Self {
        DbError::IOError { source: error }
    }
}

impl From<serde_json::Error> for DbError {
    fn from(error: serde_json::Error) -> Self {
        DbError::SerializationError { source: error }
    }
}
