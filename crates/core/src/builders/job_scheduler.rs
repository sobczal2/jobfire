use super::Builder;
use crate::{
    domain::job::scheduler::JobScheduler, managers::job_scheduler::SimpleJobScheduler,
    storage::Storage,
};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct JobSchedulerBuilder {
    inner: Arc<Mutex<JobSchedulerBuilderInner>>,
}

impl Clone for JobSchedulerBuilder {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[derive(Default)]
pub struct JobSchedulerBuilderInner {
    storage: Option<Storage>,
}

impl Builder<Arc<dyn JobScheduler>> for JobSchedulerBuilder {
    fn build(self) -> super::Result<Arc<dyn JobScheduler>> {
        let mut inner = self.inner.lock().unwrap();

        let storage = match inner.storage.take() {
            Some(element) => element,
            None => {
                return Err(super::Error::ElementMissing {
                    element_name: "storage".to_owned(),
                });
            }
        };
        let job_scheduler = SimpleJobScheduler::new(storage);
        Ok(Arc::new(job_scheduler))
    }
}

impl JobSchedulerBuilder {
    pub fn with_storage(&self, storage: impl Into<Storage>) -> Self {
        self.inner.lock().unwrap().storage.replace(storage.into());
        self.clone()
    }
}
