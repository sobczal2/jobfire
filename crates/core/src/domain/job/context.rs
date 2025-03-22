use std::sync::Arc;

use super::scheduler::JobScheduler;

/// Marker trait for context data accessible from jobs.
/// Types implementing this must be `Send` + `Sync` + `'static`.
pub trait ContextData: Send + Sync + 'static {}

/// Context for every job execution.
/// Provides access to data and allows scheduling new jobs.
pub struct Context<TData: ContextData> {
    data: Arc<TData>,
    job_scheduler: Arc<dyn JobScheduler>,
}

impl<TData: ContextData> Clone for Context<TData> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            job_scheduler: self.job_scheduler.clone(),
        }
    }
}

impl<TData: ContextData> Context<TData> {
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
