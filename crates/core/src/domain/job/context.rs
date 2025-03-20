use std::sync::Arc;

use super::scheduler::JobScheduler;

pub trait JobContextData: Send + Sync + 'static {}

pub struct JobContext<TData: JobContextData> {
    data: Arc<TData>,
    job_scheduler: Arc<dyn JobScheduler>,
}

impl<TData: JobContextData> Clone for JobContext<TData> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            job_scheduler: self.job_scheduler.clone(),
        }
    }
}

impl<TData: JobContextData> JobContext<TData> {
    pub fn new(data: Arc<TData>, job_scheduler: Arc<dyn JobScheduler>) -> Self {
        Self {
            data,
            job_scheduler,
        }
    }

    pub fn data(&self) -> Arc<TData> {
        self.data.clone()
    }
}
