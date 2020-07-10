use super::DownloadError;
use std::error::Error;
use std::fmt;

// TODO is there a way to share some of this copypasta between other errors?

#[derive(Debug)]
pub enum Md5SearchError {
    GameMismatch,
    /* Finding a mod from a different game when performing an md5 lookup could maybe happen
     * due to something the user has done. It could theoretically also mean an md5
     * collision on Nexuxmods.
     */
    HashMismatch,
    DownloadError { source: DownloadError },
}

impl Error for Md5SearchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Md5SearchError::DownloadError { source } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for Md5SearchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Md5SearchError::HashMismatch => f.write_str("HashMismatch"),
            Md5SearchError::GameMismatch => f.write_str("GameMismatch"),
            Md5SearchError::DownloadError { source } => source.fmt(f),
        }
    }
}

impl From<DownloadError> for Md5SearchError {
    fn from(error: DownloadError) -> Self {
        Md5SearchError::DownloadError { source: error }
    }
}

impl From<std::io::Error> for Md5SearchError {
    fn from(error: std::io::Error) -> Self {
        Md5SearchError::DownloadError {
            source: DownloadError::from(error),
        }
    }
}
impl From<reqwest::Error> for Md5SearchError {
    fn from(error: reqwest::Error) -> Self {
        Md5SearchError::DownloadError {
            source: DownloadError::from(error),
        }
    }
}
