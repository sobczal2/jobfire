use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("not found")]
    NotFound,
    #[error("custom: {message}")]
    Custom { message: String },
}

pub type Result<T> = std::result::Result<T, Error>;
