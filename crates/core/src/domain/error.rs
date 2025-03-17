use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("serialization failed")]
    SerializationFailed,
}

pub type Result<T> = std::result::Result<T, Error>;
