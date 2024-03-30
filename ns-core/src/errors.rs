use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Network error: {0}")]
    Network(#[from] std::net::AddrParseError),
    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::error::DecodeError),
}

pub type Result<T> = std::result::Result<T, Error>;
