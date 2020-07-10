use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DownloadError {
    ApiKeyMissing,
    IOError { source: std::io::Error },
    ConnectionError { source: reqwest::Error },
}

impl Error for DownloadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DownloadError::ApiKeyMissing => None,
            DownloadError::IOError { ref source } => Some(source),
            DownloadError::ConnectionError { ref source } => Some(source),
        }
    }
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DownloadError::ApiKeyMissing => f.write_str("ApiKeyMissing"),
            DownloadError::IOError { source } => source.fmt(f),
            DownloadError::ConnectionError { source } => source.fmt(f),
        }
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(error: std::io::Error) -> Self {
        DownloadError::IOError { source: error }
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(error: reqwest::Error) -> Self {
        DownloadError::ConnectionError { source: error }
    }
}
