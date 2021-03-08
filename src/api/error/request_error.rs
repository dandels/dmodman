use std::error::Error;
use std::fmt;
use tokio::io;

#[derive(Debug)]
pub enum RequestError {
    ApiKeyMissing,
    IOError { source: io::Error },
    ConnectionError { source: reqwest::Error },
}

impl Error for RequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RequestError::ApiKeyMissing => None,
            RequestError::IOError { ref source } => Some(source),
            RequestError::ConnectionError { ref source } => Some(source),
        }
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestError::ApiKeyMissing => {
                f.write_str("No apikey configured. API connections are disabled.")
            }
            RequestError::IOError { source } => source.fmt(f),
            RequestError::ConnectionError { source } => source.fmt(f),
        }
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
