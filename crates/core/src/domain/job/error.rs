use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Clone, Serialize, Deserialize, Debug)]
pub enum Error {
    #[error("job impl build failed")]
    JobImplBuildFailed,
    #[error("job failed: {message}")]
    Custom { message: String },
}

pub type Result<T> = std::result::Result<T, Error>;
