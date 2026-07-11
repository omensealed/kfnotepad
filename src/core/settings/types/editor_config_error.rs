#[derive(Debug)]
pub enum EditorConfigError {
    Read {
        path: PathBuf,
        source: io::Error,
    },
    Invalid {
        path: PathBuf,
        message: String,
    },
    CreateDir {
        path: PathBuf,
        source: io::Error,
    },
    CreateTemp {
        path: PathBuf,
        source: io::Error,
    },
    WriteTemp {
        path: PathBuf,
        source: io::Error,
    },
    Remove {
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

impl fmt::Display for EditorConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(formatter, "cannot read {}: {source}", path.display())
            }
            Self::Invalid { path, message } => {
                write!(formatter, "invalid config {}: {message}", path.display())
            }
            Self::CreateDir { path, source } => {
                write!(
                    formatter,
                    "cannot create config directory {}: {source}",
                    path.display()
                )
            }
            Self::CreateTemp { path, source } => {
                write!(
                    formatter,
                    "cannot create temporary config {}: {source}",
                    path.display()
                )
            }
            Self::WriteTemp { path, source } => {
                write!(
                    formatter,
                    "cannot write temporary config {}: {source}",
                    path.display()
                )
            }
            Self::Remove { path, source } => {
                write!(
                    formatter,
                    "cannot remove config {}: {source}",
                    path.display()
                )
            }
            Self::FlushTemp { path, source } => {
                write!(
                    formatter,
                    "cannot flush temporary config {}: {source}",
                    path.display()
                )
            }
            Self::Rename { from, to, source } => {
                write!(
                    formatter,
                    "cannot replace config {} with {}: {source}",
                    to.display(),
                    from.display()
                )
            }
        }
    }
}
