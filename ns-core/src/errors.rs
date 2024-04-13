use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Network error: {0}")]
    Network(#[from] std::net::AddrParseError),
    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::error::DecodeError),
    #[error("IntParse error: {0}")]
    IntParse(#[from] std::num::ParseIntError),

    #[error("Server error: {0}")]
    Server(#[from] ServerError),
}

impl<T> From<std::sync::mpsc::SendError<T>> for Error {
    fn from(_: std::sync::mpsc::SendError<T>) -> Self {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "MPSC send error",
        ))
    }
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("User {} already connected", .0)]
    UserAlreadyConnected(String),
}

pub type Result<T> = std::result::Result<T, Error>;
