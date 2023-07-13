use crate::cache::CacheError;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use tokio::io;
use tokio::task::JoinError;
use tokio_tungstenite::tungstenite;
use url::ParseError;

#[derive(Debug)]
pub enum ApiError {
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
    WebsocketError { source: tungstenite::Error },
}

impl Error for ApiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ApiError::ConnectionError { ref source } => Some(source),
            ApiError::CacheError { ref source } => Some(source),
            ApiError::Expired => None,
            ApiError::IOError { ref source } => Some(source),
            ApiError::JoinError { ref source } => Some(source),
            ApiError::ParseError { ref source } => Some(source),
            ApiError::ParseIntError { ref source } => Some(source),
            ApiError::SerializationError { ref source } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::ApiKeyMissing => f.write_str("No apikey configured. API connections are disabled."),
            ApiError::CacheError { source } => source.fmt(f),
            ApiError::ConnectionError { source } => source.fmt(f),
            ApiError::Expired => f.write_str("Download link is expired."),
            ApiError::IOError { source } => source.fmt(f),
            ApiError::JoinError { source } => source.fmt(f),
            ApiError::SerializationError { source } => source.fmt(f),
            ApiError::IsUnitTest => f.write_str("Unit tests aren't allowed to make network connections."),
            ApiError::ParseError { source } => source.fmt(f),
            ApiError::ParseIntError { source } => source.fmt(f),
            ApiError::WebsocketError { source } => source.fmt(f),
        }
    }
}

impl From<JoinError> for ApiError {
    fn from(error: JoinError) -> Self {
        ApiError::JoinError { source: error }
    }
}

impl From<CacheError> for ApiError {
    fn from(error: CacheError) -> Self {
        ApiError::CacheError { source: error }
    }
}

impl From<io::Error> for ApiError {
    fn from(error: io::Error) -> Self {
        ApiError::IOError { source: error }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(error: reqwest::Error) -> Self {
        ApiError::ConnectionError { source: error }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        ApiError::SerializationError { source: error }
    }
}

impl From<ParseError> for ApiError {
    fn from(error: ParseError) -> Self {
        ApiError::ParseError { source: error }
    }
}

impl From<ParseIntError> for ApiError {
    fn from(error: ParseIntError) -> Self {
        ApiError::ParseIntError { source: error }
    }
}

impl From<tungstenite::Error> for ApiError {
    fn from(error: tungstenite::Error) -> Self {
        ApiError::WebsocketError { source: error }
    }
}
