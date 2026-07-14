//! Errors produced by conservative atomic text-file saves.

use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum SaveError {
    Metadata {
        path: PathBuf,
        source: io::Error,
    },
    Directory {
        path: PathBuf,
    },
    Symlink {
        path: PathBuf,
    },
    NotRegular {
        path: PathBuf,
    },
    ExternalModification {
        path: PathBuf,
    },
    ExternalTargetTooLarge {
        path: PathBuf,
        bytes: u64,
        limit: u64,
    },
    TooLarge {
        path: PathBuf,
        bytes: u64,
        limit: u64,
    },
    CreateTemp {
        path: PathBuf,
        source: io::Error,
    },
    WriteTemp {
        path: PathBuf,
        source: io::Error,
    },
    FlushTemp {
        path: PathBuf,
        source: io::Error,
    },
    Rename {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },
}

impl fmt::Display for SaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Metadata { path, source } => {
                write!(
                    formatter,
                    "cannot inspect {} before save: {source}",
                    path.display()
                )
            }
            SaveError::Directory { path } => {
                write!(formatter, "cannot save over directory {}", path.display())
            }
            SaveError::Symlink { path } => {
                write!(
                    formatter,
                    "refusing to save through symlink {}",
                    path.display()
                )
            }
            SaveError::NotRegular { path } => {
                write!(
                    formatter,
                    "refusing to save over non-regular file {}",
                    path.display()
                )
            }
            SaveError::ExternalModification { path } => {
                write!(
                    formatter,
                    "file changed on disk since open or last save: {}; reload or save as a new file",
                    path.display()
                )
            }
            SaveError::ExternalTargetTooLarge { path, bytes, limit } => {
                write!(
                    formatter,
                    "file on disk is too large to validate before save: {} is at least {bytes} bytes and exceeds {limit} bytes; reload or save as a new file",
                    path.display()
                )
            }
            SaveError::TooLarge { path, bytes, limit } => {
                write!(
                    formatter,
                    "cannot save {}: {bytes} bytes exceeds {limit} bytes",
                    path.display()
                )
            }
            SaveError::CreateTemp { path, source } => {
                write!(
                    formatter,
                    "cannot create temporary file {}: {source}",
                    path.display()
                )
            }
            SaveError::WriteTemp { path, source } => {
                write!(
                    formatter,
                    "cannot write temporary file {}: {source}",
                    path.display()
                )
            }
            SaveError::FlushTemp { path, source } => {
                write!(
                    formatter,
                    "cannot flush temporary file {}: {source}",
                    path.display()
                )
            }
            SaveError::Rename { from, to, source } => {
                write!(
                    formatter,
                    "cannot replace {} with {}: {source}",
                    to.display(),
                    from.display()
                )
            }
        }
    }
}
