use super::{Md5SearchError, RequestError};
use crate::cache::CacheError;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use tokio::io;
use url::ParseError;

#[derive(Debug)]
pub enum DownloadError {
    CacheError { source: CacheError },
    Expired,
    IOError { source: io::Error },
    RequestError { source: RequestError },
    Md5SearchError { source: Md5SearchError },
    ParseError { source: ParseError },
    ParseIntError { source: ParseIntError },
}

// TODO: is there a way to get rid of this mindless boilerplate?
impl Error for DownloadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DownloadError::CacheError { ref source } => Some(source),
            DownloadError::Expired => None,
            DownloadError::IOError { ref source } => Some(source),
            DownloadError::Md5SearchError { ref source } => Some(source),
            DownloadError::ParseError { ref source } => Some(source),
            DownloadError::ParseIntError { ref source } => Some(source),
            DownloadError::RequestError { ref source } => Some(source),
        }
    }
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            DownloadError::CacheError { source } => source.fmt(f),
            DownloadError::Expired => f.write_str("Download link is expired."),
            DownloadError::IOError { source } => source.fmt(f),
            DownloadError::Md5SearchError { source } => source.fmt(f),
            DownloadError::ParseError { source } => source.fmt(f),
            DownloadError::ParseIntError { source } => source.fmt(f),
            DownloadError::RequestError { source } => source.fmt(f),
        }
    }
}

impl From<io::Error> for DownloadError {
    fn from(error: io::Error) -> Self {
        DownloadError::IOError { source: error }
    }
}

impl From<RequestError> for DownloadError {
    fn from(error: RequestError) -> Self {
        DownloadError::RequestError { source: error }
    }
}

impl From<Md5SearchError> for DownloadError {
    fn from(error: Md5SearchError) -> Self {
        DownloadError::Md5SearchError { source: error }
    }
}

impl From<ParseError> for DownloadError {
    fn from(error: ParseError) -> Self {
        DownloadError::ParseError { source: error }
    }
}

impl From<ParseIntError> for DownloadError {
    fn from(error: ParseIntError) -> Self {
        DownloadError::ParseIntError { source: error }
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(error: reqwest::Error) -> Self {
        DownloadError::RequestError {
            source: RequestError::ConnectionError { source: error },
        }
    }
}

impl From<CacheError> for DownloadError {
    fn from(error: CacheError) -> Self {
        DownloadError::CacheError { source: error }
    }
}
