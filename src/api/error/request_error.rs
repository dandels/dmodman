use crate::cache::CacheError;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use tokio::io;
use tokio::task::JoinError;
use url::ParseError;

#[derive(Debug)]
pub enum RequestError {
    ApiKeyMissing,
    ConnectionError { source: reqwest::Error },
    CacheError { source: CacheError },
    Expired,
    IOError { source: io::Error },
    IsUnitTest,
    JoinError { source: JoinError },
    ParseError { source: ParseError },
    ParseIntError { source: ParseIntError },
    SerializationError { source: serde_json::Error },
}

impl Error for RequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RequestError::ConnectionError { ref source } => Some(source),
            RequestError::CacheError { ref source } => Some(source),
            RequestError::Expired => None,
            RequestError::IOError { ref source } => Some(source),
            RequestError::JoinError { ref source } => Some(source),
            RequestError::ParseError { ref source } => Some(source),
            RequestError::ParseIntError { ref source } => Some(source),
            RequestError::SerializationError { ref source } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestError::ApiKeyMissing => f.write_str("No apikey configured. API connections are disabled."),
            RequestError::CacheError { source } => source.fmt(f),
            RequestError::ConnectionError { source } => source.fmt(f),
            RequestError::Expired => f.write_str("Download link is expired."),
            RequestError::IOError { source } => source.fmt(f),
            RequestError::JoinError { source } => source.fmt(f),
            RequestError::SerializationError { source } => source.fmt(f),
            RequestError::IsUnitTest => f.write_str("Unit tests aren't allowed to make network connections."),
            RequestError::ParseError { source } => source.fmt(f),
            RequestError::ParseIntError { source } => source.fmt(f),
        }
    }
}

impl From<JoinError> for RequestError {
    fn from(error: JoinError) -> Self {
        RequestError::JoinError { source: error }
    }
}

impl From<CacheError> for RequestError {
    fn from(error: CacheError) -> Self {
        RequestError::CacheError { source: error }
    }
}

impl From<io::Error> for RequestError {
    fn from(error: io::Error) -> Self {
        RequestError::IOError { source: error }
    }
}

impl From<reqwest::Error> for RequestError {
    fn from(error: reqwest::Error) -> Self {
        RequestError::ConnectionError { source: error }
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(error: serde_json::Error) -> Self {
        RequestError::SerializationError { source: error }
    }
}

impl From<ParseError> for RequestError {
    fn from(error: ParseError) -> Self {
        RequestError::ParseError { source: error }
    }
}

impl From<ParseIntError> for RequestError {
    fn from(error: ParseIntError) -> Self {
        RequestError::ParseIntError { source: error }
    }
}
