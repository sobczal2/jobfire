use thiserror::Error;

use crate::storage;

#[derive(Debug, Error)]
pub enum Error {
    #[error("stop failed")]
    StopFailed,
    #[error("storage error: {0}")]
    Storage(#[from] storage::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
