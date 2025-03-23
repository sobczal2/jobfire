use super::{Builder, job_actions_registry::JobActionsRegistryBuilder};
use crate::{
    domain::job::{
        context::{Context, ContextData},
        r#impl::JobImpl,
        scheduler::JobScheduler,
    },
    managers::job_manager::JobManager,
    runners::job::JobRunner,
    services::{Services, builder::ServicesBuilder},
    storage::Storage,
    workers::job::JobWorkerSettings,
};
use std::sync::{Arc, Mutex};

pub struct JobManagerBuilder<TData: ContextData> {
    inner: Arc<Mutex<JobfireManagerBuilderInner<TData>>>,
}

impl<TData: ContextData> Clone for JobManagerBuilder<TData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TData: ContextData> Default for JobManagerBuilder<TData> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

pub type JobSchedulerFactory = Box<dyn FnOnce(Storage) -> Box<dyn JobScheduler>>;

pub struct JobfireManagerBuilderInner<TData: ContextData> {
    storage: Option<Storage>,
    services: ServicesBuilder,
    context_data: Option<TData>,
    job_scheduler_factory: Option<JobSchedulerFactory>,
    job_actions_registry_builder: JobActionsRegistryBuilder<TData>,
    job_worker_settings: Option<JobWorkerSettings>,
}

impl<TData: ContextData> Default for JobfireManagerBuilderInner<TData> {
    fn default() -> Self {
        Self {
            storage: Default::default(),
            services: ServicesBuilder::default(),
            context_data: Default::default(),
            job_scheduler_factory: Default::default(),
            job_actions_registry_builder: JobActionsRegistryBuilder::default(),
            job_worker_settings: Default::default(),
        }
    }
}

macro_rules! check_element {
    ($inner:expr, $element:ident) => {
        match $inner.$element.take() {
            Some(element) => element,
            None => {
                return Err(super::Error::ElementMissing {
                    element_name: stringify!($element).to_owned(),
                });
            }
        }
    };
}

impl<TData: ContextData> Builder<JobManager<TData>> for JobManagerBuilder<TData> {
    fn build(self) -> super::Result<JobManager<TData>> {
        let mut inner = self.inner.lock().unwrap();

        let storage = check_element!(inner, storage);
        let context_data = Arc::from(check_element!(inner, context_data));
        let job_scheduler_factory = check_element!(inner, job_scheduler_factory);
        let job_worker_settings = check_element!(inner, job_worker_settings);
        let job_actions_registry_builder = inner.job_actions_registry_builder.clone();

        let services: Services = inner.services.clone().into();
        let job_scheduler: Arc<dyn JobScheduler> = (job_scheduler_factory)(storage.clone()).into();
        let context = Context::new(context_data, job_scheduler.clone(), services.clone());
        let job_actions_registry = job_actions_registry_builder.build()?;
        let job_runner = JobRunner::new(
            storage.clone(),
            context.clone(),
            job_actions_registry.clone(),
        );

        let manager = JobManager::start(
            context,
            services,
            storage,
            job_runner,
            job_worker_settings,
            job_scheduler,
        );
        Ok(manager)
    }
}

impl<TData: ContextData> JobManagerBuilder<TData> {
    pub fn with_storage(&self, storage: impl Into<Storage>) -> Self {
        self.inner.lock().unwrap().storage.replace(storage.into());
        self.clone()
    }

    pub fn with_context_data(&self, context_data: TData) -> Self {
        self.inner
            .lock()
            .unwrap()
            .context_data
            .replace(context_data);
        self.clone()
    }

    pub fn with_job_scheduler_factory(
        &self,
        job_scheduler_factory: Box<dyn FnOnce(Storage) -> Box<dyn JobScheduler>>,
    ) -> Self {
        self.inner
            .lock()
            .unwrap()
            .job_scheduler_factory
            .replace(job_scheduler_factory);
        self.clone()
    }

    pub fn with_job_worker_settings(&self, job_worker_settings: JobWorkerSettings) -> Self {
        self.inner
            .lock()
            .unwrap()
            .job_worker_settings
            .replace(job_worker_settings);
        self.clone()
    }

    pub fn register_job_impl<TJobImpl: JobImpl<TData>>(&self) -> Self {
        self.inner
            .lock()
            .unwrap()
            .job_actions_registry_builder
            .register::<TJobImpl>();
        self.clone()
    }

    pub fn add_service<T: 'static + Send + Sync>(&self, service: T) -> Self {
        self.inner.lock().unwrap().services.add_service(service);
        self.clone()
    }
}
