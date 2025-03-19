use async_trait::async_trait;
use getset::Getters;

use crate::domain::job::{context::JobContextData, pending::PendingJob};

#[derive(Getters)]
#[getset(get = "pub")]
pub struct JobRunnerInput {
    pending_job: PendingJob,
}

impl JobRunnerInput {
    pub fn new(pending_job: PendingJob) -> Self {
        Self { pending_job }
    }
}

#[async_trait]
pub trait JobRunner<TData: JobContextData>: Send + Sync + 'static {
    async fn run(&self, input: &JobRunnerInput);
}
