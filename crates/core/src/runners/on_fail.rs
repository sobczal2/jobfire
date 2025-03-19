use async_trait::async_trait;
use getset::Getters;

use crate::domain::job::{context::JobContextData, pending::PendingJob};

#[derive(Getters)]
#[getset(get = "pub")]
pub struct OnFailRunnerInput {
    pending_job: PendingJob,
}

impl OnFailRunnerInput {
    pub fn new(pending_job: PendingJob) -> Self {
        Self { pending_job }
    }
}

#[async_trait]
pub trait OnFailRunner<TData: JobContextData>: Send + Sync + 'static {
    async fn run(&self, on_fail_runner_input: &OnFailRunnerInput);
}
