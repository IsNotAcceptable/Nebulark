use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("tunnel error: {0}")]
    Tunnel(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("platform error: {0}")]
    Platform(String),
    #[error("AWG protocol error: {0}")]
    Awg(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;