pub mod domain;
pub mod managers;
pub mod storage;
pub mod workers;

pub use async_trait::async_trait;
pub use uuid::Uuid;

pub mod prelude {
    pub use super::Uuid;
    pub use super::async_trait;
    pub use super::domain::{
        Error, FailedJob, JobContext, JobId, JobImpl, PendingJob, Report, RunningJob, SuccessfulJob,
    };
    pub use super::managers::JobfireManager;
    pub use super::storage::{
        FailedJobRepo, PendingJobRepo, RunningJobRepo, Storage, SuccessfulJobRepo,
    };
}
