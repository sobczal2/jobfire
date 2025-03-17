use async_trait::async_trait;
use getset::Getters;

use crate::domain::job::{JobContext, PendingJob};

#[derive(Getters)]
#[getset(get = "pub")]
pub struct OnSuccessRunnerInput {
    pending_job: PendingJob,
}

impl OnSuccessRunnerInput {
    pub fn new(pending_job: PendingJob) -> Self {
        Self { pending_job }
    }
}

#[async_trait]
pub trait OnSuccessRunner<TJobContext: JobContext>: Send + Sync + 'static {
    async fn run(&self, on_success_runner_input: &OnSuccessRunnerInput);
}
