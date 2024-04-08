use thiserror::Error;

use crate::models::canvas::CanvasEntry;

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
    #[error("MPSC error: {0}")]
    Mpsc(#[from] std::sync::mpsc::SendError<CanvasEntry>),
}

pub type Result<T> = std::result::Result<T, Error>;
