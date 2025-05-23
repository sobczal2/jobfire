use chrono::{DateTime, Utc};
use context::ContextData;
use getset::Getters;
use id::JobId;
use r#impl::{JobImpl, SerializedJobImpl};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod context;
pub mod error;
pub mod id;
pub mod r#impl;
pub mod pending;
pub mod report;
pub mod running;

#[derive(Error, Debug)]
pub enum Error {
    #[error("building job failed")]
    BuildingJobFailed,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Job information with serialized implementation.
///
/// This structure represents a complete job definition that can be stored
/// in a persistent storage. It contains all the information needed to
/// recreate and execute the job at runtime.
#[derive(Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Job {
    /// Unique identifier for this job.
    ///
    /// Used to reference this job across different components of the system.
    id: JobId,

    /// Timestamp when the job was initially created.
    ///
    /// Used for tracking job lifecycle and potentially for metrics.
    created_at: DateTime<Utc>,

    /// Serialized implementation of the job's functionality.
    ///
    /// Contains the serialized form of the job logic. At runtime,
    /// the actual job functionality is recreated from this serialized data.
    r#impl: SerializedJobImpl,
}

impl Job {
    fn new(id: JobId, created_at: DateTime<Utc>, r#impl: SerializedJobImpl) -> Self {
        Self {
            id,
            created_at,
            r#impl,
        }
    }

    /// Function to create a job from custom job implementation
    pub fn from_impl<TData: ContextData>(job_impl: impl JobImpl<TData>) -> Result<Self> {
        Ok(Self::new(
            JobId::default(),
            Utc::now(),
            SerializedJobImpl::from_job_impl(r#job_impl).map_err(|_| Error::BuildingJobFailed)?,
        ))
    }
}
