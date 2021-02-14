use super::{RequestError, Md5SearchError};
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use url::ParseError;

#[derive(Debug)]
pub enum DownloadError {
    Expired,
    RequestError { source: RequestError },
    Md5SearchError { source: Md5SearchError },
    ParseError { source: ParseError },
    ParseIntError { source: ParseIntError },
}

// TODO: is there a way to get rid of this mindless boilerplate?
impl Error for DownloadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DownloadError::RequestError { ref source } => Some(source),
            DownloadError::Md5SearchError { ref source } => Some(source),
            DownloadError::ParseError { ref source } => Some(source),
            DownloadError::ParseIntError { ref source } => Some(source),
            DownloadError::Expired => None,
        }
    }
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            DownloadError::Expired => f.write_str("Expired"),
            DownloadError::RequestError { source } => source.fmt(f),
            DownloadError::Md5SearchError { source } => source.fmt(f),
            DownloadError::ParseError { source } => source.fmt(f),
            DownloadError::ParseIntError { source } => source.fmt(f),
        }
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