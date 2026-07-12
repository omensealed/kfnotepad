//! Errors produced while validating and opening a text file.

use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum OpenError {
    Access {
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
    TooLarge {
        path: PathBuf,
        bytes: u64,
        limit: u64,
    },
    ReadUtf8 {
        path: PathBuf,
        source: io::Error,
    },
}

impl fmt::Display for OpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpenError::Access { path, source } => {
                write!(formatter, "cannot access {}: {source}", path.display())
            }
            OpenError::Directory { path } => {
                write!(
                    formatter,
                    "{} is a directory, not a text file",
                    path.display()
                )
            }
            OpenError::Symlink { path } => {
                write!(formatter, "refusing to open symlink {}", path.display())
            }
            OpenError::NotRegular { path } => {
                write!(
                    formatter,
                    "refusing to open non-regular file {}",
                    path.display()
                )
            }
            OpenError::TooLarge { path, bytes, limit } => {
                write!(
                    formatter,
                    "{} is too large to open safely: {bytes} bytes exceeds {limit} bytes",
                    path.display()
                )
            }
            OpenError::ReadUtf8 { path, source } => {
                write!(
                    formatter,
                    "cannot read {} as UTF-8 text: {source}",
                    path.display()
                )
            }
        }
    }
}
