//use async_zip::error::ZipError;
//use sevenz_rust::Error as SevenzError;
use std::error::Error;
use std::fmt;
//use unrar::error::UnrarError as RarError;
use super::libarchive::ArchiveError;
//use compress_tools::Error as DecompressError;

#[derive(Debug)]
pub enum InstallError {
    AlreadyExists,
    ArchiveDeleted,
    ArchiveError { source: ArchiveError },
    //DecompressError { source: DecompressError },
    //RarError { source: RarError },
    //ZipError { source: ZipError },
    //SevenzError { source: SevenzError },
    InProgress,
    IO { source: std::io::Error },
}

impl Error for InstallError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InstallError::AlreadyExists => None,
            InstallError::ArchiveDeleted => None,
            InstallError::ArchiveError { ref source } => Some(source),
            //InstallError::SevenzError { ref source } => Some(source),
            //InstallError::RarError { ref source } => Some(source),
            //InstallError::ZipError { ref source } => Some(source),
            InstallError::InProgress => None,
            InstallError::IO { ref source } => Some(source),
        }
    }
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InstallError::AlreadyExists => f.write_str("Target directory already exists."),
            InstallError::ArchiveDeleted => f.write_str("Archive no longer exists."),
            InstallError::ArchiveError { source } => source.fmt(f),
            //InstallError::SevenzError { source } => source.fmt(f),
            //InstallError::RarError { source } => source.fmt(f),
            //InstallError::ZipError { source } => source.fmt(f),
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

impl From<ArchiveError> for InstallError {
    fn from(source: ArchiveError) -> Self {
        Self::ArchiveError { source }
    }
}

//impl From<SevenzError> for InstallError {
//    fn from(source: SevenzError) -> Self {
//        Self::SevenzError { source }
//    }
//}
//
//impl From<RarError> for InstallError {
//    fn from(source: RarError) -> Self {
//        Self::RarError { source }
//    }
//}
//
//impl From<ZipError> for InstallError {
//    fn from(source: ZipError) -> Self {
//        Self::ZipError { source }
//    }
//}
