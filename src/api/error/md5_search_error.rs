use super::RequestError;
use std::error::Error;
use std::fmt;
use tokio::io;

// TODO is there a way to share some of this copypasta between other errors?

#[derive(Debug)]
#[allow(dead_code)]
pub enum Md5SearchError {
    /* Finding a mod from a different game when performing an md5 lookup could maybe happen
     * due to something the user has done. It could theoretically also mean an md5
     * collision on Nexuxmods.
     */
    GameMismatch,
    HashMismatch,
    RequestError { source: RequestError },
}

impl Error for Md5SearchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Md5SearchError::RequestError { source } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for Md5SearchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Md5SearchError::HashMismatch => f.write_str("HashMismatch"),
            Md5SearchError::GameMismatch => f.write_str("GameMismatch"),
            Md5SearchError::RequestError { source } => source.fmt(f),
        }
    }
}

impl From<RequestError> for Md5SearchError {
    fn from(error: RequestError) -> Self {
        Md5SearchError::RequestError { source: error }
    }
}

impl From<io::Error> for Md5SearchError {
    fn from(error: io::Error) -> Self {
        Md5SearchError::RequestError {
            source: RequestError::from(error),
        }
    }
}
impl From<reqwest::Error> for Md5SearchError {
    fn from(error: reqwest::Error) -> Self {
        Md5SearchError::RequestError {
            source: RequestError::from(error),
        }
    }
}
