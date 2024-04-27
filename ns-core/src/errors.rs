use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Network error: {0}")]
    AddrParseError(#[from] std::net::AddrParseError),
    #[error("Decode error: {0}")]
    DecodeError(#[from] bincode::error::DecodeError),
    #[error("Encode error: {0}")]
    EncodeError(#[from] bincode::error::EncodeError),
    #[error("IntParse error: {0}")]
    IntParseError(#[from] std::num::ParseIntError),
    #[error("Server error: {:?}", .0)]
    ServerError(#[from] ServerError),
}

#[derive(Debug, Error)]
pub enum ServerError {
    UsernameTaken(String),
    UserNotFound,
    LockError,
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::UsernameTaken(name) => {
                write!(f, "Username {} is already taken", name)
            }
            ServerError::UserNotFound => {
                write!(f, "User not found")
            }
            ServerError::LockError => {
                write!(f, "Failed to lock server state")
            }
        }
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for Error {
    fn from(_: std::sync::mpsc::SendError<T>) -> Self {
        Error::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "MPSC send error",
        ))
    }
}
