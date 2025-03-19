use async_trait::async_trait;
use getset::Getters;

use crate::domain::job::{context::JobContextData, pending::PendingJob};

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
pub trait OnSuccessRunner<TData: JobContextData>: Send + Sync + 'static {
    async fn run(&self, input: &OnSuccessRunnerInput);
}
