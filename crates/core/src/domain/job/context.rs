use std::sync::Arc;

use super::scheduler::JobScheduler;

pub trait JobContextData: Sized + Clone + Send + Sync + 'static {}

#[derive(Clone)]
pub struct JobContext<TData: JobContextData> {
    data: TData,
    job_scheduler: Arc<dyn JobScheduler>,
}

impl<TData: JobContextData> JobContext<TData> {
    pub fn new(data: TData, job_scheduler: Arc<dyn JobScheduler>) -> Self {
        Self {
            data,
            job_scheduler,
        }
    }
}
