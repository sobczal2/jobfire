use std::sync::Arc;

use getset::Getters;
use job::{FailedJobRepo, PendingJobRepo, RunningJobRepo, SuccessfullJobRepo};

use crate::domain::job::Context;

pub(crate) mod error;
pub(crate) mod job;

#[derive(Clone, Getters)]
#[getset(get = "pub")]
pub struct Persistence<T: Context> {
    pending_job_repo: Arc<dyn PendingJobRepo<T>>,
    running_job_repo: Arc<dyn RunningJobRepo<T>>,
    failed_job_repo: Arc<dyn FailedJobRepo<T>>,
    successful_job_repo: Arc<dyn SuccessfullJobRepo<T>>,
}
