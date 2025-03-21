use chrono::{DateTime, Utc};
use context::JobContextData;
use getset::Getters;
use id::JobId;
use r#impl::{JobImpl, SerializedJobImpl};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod context;
pub mod error;
pub mod failed;
pub mod id;
pub mod r#impl;
pub mod pending;
pub mod report;
pub mod running;
pub mod scheduler;
pub mod successful;

#[derive(Error, Debug)]
pub enum Error {
    #[error("building job failed")]
    BuildingJobFailed,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Job {
    id: JobId,
    created_at: DateTime<Utc>,
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

    pub fn from_impl<TData: JobContextData>(job_impl: impl JobImpl<TData>) -> Result<Self> {
        Ok(Self::new(
            JobId::new(),
            Utc::now(),
            SerializedJobImpl::from_job_impl(r#job_impl).map_err(|_| Error::BuildingJobFailed)?,
        ))
    }
}
