use crate::{api::RequestError, cache::error::*};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum UpdateError {
    CacheError { source: CacheError },
    RequestError { source: RequestError },
}

impl Error for UpdateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            UpdateError::CacheError { ref source } => Some(source),
            UpdateError::RequestError { ref source } => Some(source),
        }
    }
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            UpdateError::CacheError { source } => source.fmt(f),
            UpdateError::RequestError { source } => source.fmt(f),
        }
    }
}

impl From<CacheError> for UpdateError {
    fn from(error: CacheError) -> Self {
        UpdateError::CacheError { source: error }
    }
}

impl From<RequestError> for UpdateError {
    fn from(error: RequestError) -> Self {
        UpdateError::RequestError { source: error }
    }
}
