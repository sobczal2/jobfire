use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to stop")]
    StopFailed,
    #[error("invalid settings: {0}")]
    InvalidSettings(String),
}

pub type Result<T> = std::result::Result<T, Error>;
