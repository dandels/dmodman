use super::{DownloadError, Md5SearchError};
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use url::ParseError;

#[derive(Debug)]
pub enum NxmDownloadError {
    Expired,
    DownloadError { source: DownloadError },
    Md5SearchError { source: Md5SearchError },
    ParseError { source: ParseError },
    ParseIntError { source: ParseIntError },
}

// TODO: is there a way to get rid of this mindless boilerplate?
impl Error for NxmDownloadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            NxmDownloadError::DownloadError { ref source } => Some(source),
            NxmDownloadError::Md5SearchError { ref source } => Some(source),
            NxmDownloadError::ParseError { ref source } => Some(source),
            NxmDownloadError::ParseIntError { ref source } => Some(source),
            NxmDownloadError::Expired => None,
        }
    }
}

impl fmt::Display for NxmDownloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            NxmDownloadError::Expired => f.write_str("Expired"),
            NxmDownloadError::DownloadError { source } => source.fmt(f),
            NxmDownloadError::Md5SearchError { source } => source.fmt(f),
            NxmDownloadError::ParseError { source } => source.fmt(f),
            NxmDownloadError::ParseIntError { source } => source.fmt(f),
        }
    }
}

impl From<DownloadError> for NxmDownloadError {
    fn from(error: DownloadError) -> Self {
        NxmDownloadError::DownloadError { source: error }
    }
}

impl From<Md5SearchError> for NxmDownloadError {
    fn from(error: Md5SearchError) -> Self {
        NxmDownloadError::Md5SearchError { source: error }
    }
}

impl From<ParseError> for NxmDownloadError {
    fn from(error: ParseError) -> Self {
        NxmDownloadError::ParseError { source: error }
    }
}

impl From<ParseIntError> for NxmDownloadError {
    fn from(error: ParseIntError) -> Self {
        NxmDownloadError::ParseIntError { source: error }
    }
}
