use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("serialization failed")]
    SerializationFailed,
    #[error("failed to build job impl")]
    BuildJobImplFailed,
}

pub type Result<T> = std::result::Result<T, Error>;
