use std::fmt;

#[derive(Debug)]
pub enum AppError {
    /// ISO mount/unmount failures
    Iso(String),
    /// wimlib operation failures
    Wim { code: i32, message: String },
    /// WIM file not found on mounted drive
    WimNotFound,
    /// PE parsing / PDB download failures
    Pdb(String),
    /// Generic I/O error
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Iso(msg) => write!(f, "ISO error: {msg}"),
            AppError::Wim { code, message } => write!(f, "wimlib error ({code}): {message}"),
            AppError::WimNotFound => write!(f, "no install.wim or boot.wim found on mounted ISO"),
            AppError::Pdb(msg) => write!(f, "PDB error: {msg}"),
            AppError::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e)
    }
}
