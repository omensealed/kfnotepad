#[derive(Debug)]
pub enum ManagedNotesError {
    MissingDataHome,
    InvalidNoteName,
    InvalidNotePath { path: PathBuf, message: String },
    CreateNotesDir { path: PathBuf, source: io::Error },
    CreateNote { path: PathBuf, source: SaveError },
    OpenNote { path: PathBuf, source: OpenError },
    ListNotesDir { path: PathBuf, source: io::Error },
    InspectNote { path: PathBuf, source: io::Error },
    RemoveNote { path: PathBuf, source: io::Error },
}

impl fmt::Display for ManagedNotesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManagedNotesError::MissingDataHome => {
                write!(formatter, "cannot resolve managed notes directory")
            }
            ManagedNotesError::InvalidNoteName => write!(formatter, "invalid managed note name"),
            ManagedNotesError::InvalidNotePath { path, message } => {
                write!(
                    formatter,
                    "invalid managed note path {}: {message}",
                    path.display()
                )
            }
            ManagedNotesError::CreateNotesDir { path, source } => {
                write!(
                    formatter,
                    "cannot create managed notes directory {}: {source}",
                    path.display()
                )
            }
            ManagedNotesError::CreateNote { path, source } => {
                write!(
                    formatter,
                    "cannot create managed note {}: {source}",
                    path.display()
                )
            }
            ManagedNotesError::OpenNote { path, source } => {
                write!(
                    formatter,
                    "cannot open managed note {}: {source}",
                    path.display()
                )
            }
            ManagedNotesError::ListNotesDir { path, source } => {
                write!(
                    formatter,
                    "cannot list managed notes directory {}: {source}",
                    path.display()
                )
            }
            ManagedNotesError::InspectNote { path, source } => {
                write!(
                    formatter,
                    "cannot inspect managed note entry {}: {source}",
                    path.display()
                )
            }
            ManagedNotesError::RemoveNote { path, source } => {
                write!(
                    formatter,
                    "cannot delete managed note {}: {source}",
                    path.display()
                )
            }
        }
    }
}
