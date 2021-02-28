use crate::{api::RequestError, db::error::*};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum UpdateError {
    DbError { source: DbError },
    RequestError { source: RequestError },
}

impl Error for UpdateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            UpdateError::DbError { ref source } => Some(source),
            UpdateError::RequestError { ref source } => Some(source),
        }
    }
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            UpdateError::DbError { source } => source.fmt(f),
            UpdateError::RequestError { source } => source.fmt(f),
        }
    }
}

impl From<DbError> for UpdateError {
    fn from(error: DbError) -> Self {
        UpdateError::DbError { source: error }
    }
}

impl From<RequestError> for UpdateError {
    fn from(error: RequestError) -> Self {
        UpdateError::RequestError { source: error }
    }
}
