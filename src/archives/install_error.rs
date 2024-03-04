use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum InstallError {
    AlreadyExists,
    Compression { source: compress_tools::Error },
    InProgress,
    IO { source: std::io::Error },
}

impl Error for InstallError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InstallError::AlreadyExists => None,
            InstallError::Compression { ref source } => Some(source),
            InstallError::InProgress => None,
            InstallError::IO { ref source } => Some(source),
        }
    }
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InstallError::AlreadyExists => f.write_str("Target directory already exists"),
            InstallError::Compression { source } => source.fmt(f),
            InstallError::InProgress => f.write_str("Extracting to target directory is already in progress."),
            InstallError::IO { source } => source.fmt(f),
        }
    }
}

impl From<std::io::Error> for InstallError {
    fn from(source: std::io::Error) -> Self {
        Self::IO { source }
    }
}

impl From<compress_tools::Error> for InstallError {
    fn from(source: compress_tools::Error) -> Self {
        Self::Compression { source }
    }
}
