use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Possible errors returned from job run.
#[derive(Error, Clone, Serialize, Deserialize, Debug)]
pub enum JobError {
    /// Failed to build a job, this can happen when JobImpl is not found in regsitry
    #[error("job impl build failed")]
    JobImplBuildFailed,
    /// Job has been cancelled
    #[error("job has been cancelled")]
    JobCancelled,
    /// Policy stopped
    #[error("job has been stopped by policy")]
    PolicyShortCircuit,
    /// custom message reserved for user defined errors
    #[error("job failed: {message}")]
    Custom { message: String },
}

pub type JobResult<T> = std::result::Result<T, JobError>;
