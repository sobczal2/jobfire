use crate::{
    domain::job::{
        Job,
        context::{Context, ContextData},
        id::JobId,
        r#impl::JobImpl,
    },
    runners::{job::JobRunner, on_fail::OnFailRunner, on_success::OnSuccessRunner},
    services::{Services, verify::ServiceMissing},
    storage::{self, Storage},
    util::r#async::poll_predicate,
    verify_services,
    workers::job::{JobWorker, JobWorkerHandle, JobWorkerSettings, State},
};
use chrono::{DateTime, Duration, Utc};
use thiserror::Error;
use tokio::time::interval;

use super::job_scheduler::{self, JobScheduler};

#[derive(Error, Debug)]
pub enum Error {
    #[error("stop failed")]
    StopFailed,
    #[error("storage error: {0}")]
    Storage(#[from] storage::error::Error),
    #[error("scheduler error: {0}")]
    Scheduler(#[from] job_scheduler::Error),
    #[error("failed to build a job")]
    JobBuildFailed,
    #[error("service missing: {0}")]
    ServiceMissing(String),
    #[error("internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
pub struct JobManager<TData: ContextData> {
    context: Context<TData>,
    job_worker_handle: JobWorkerHandle,
}

impl<TData: ContextData> JobManager<TData> {
    pub fn new_default<B>(data: TData, builder: B) -> std::result::Result<Self, ServiceMissing>
    where
        B: FnOnce(&Services),
    {
        let services = Services::default();
        let context = Context::new(data, services.clone());
        add_default_services(&context);
        builder(&services);
        Self::verify(&services)?;
        services.verify()?;

        Ok(Self::start(context))
    }

    pub fn new_empty<B>(data: TData, builder: B) -> std::result::Result<Self, ServiceMissing>
    where
        B: FnOnce(&Services),
    {
        let services = Services::default();
        let context = Context::new(data, services.clone());
        builder(&services);
        Self::verify(&services)?;
        services.verify()?;

        Ok(Self::start(context))
    }

    fn verify(services: &Services) -> std::result::Result<(), ServiceMissing> {
        verify_services!(
            services,
            JobWorkerSettings,
            Storage,
            JobRunner<TData>,
            JobScheduler
        );

        Ok(())
    }

    fn start(context: Context<TData>) -> Self {
        let job_worker_settings = context.get_required_service::<JobWorkerSettings>();
        let storage = context.get_required_service::<Storage>();
        let job_runner = context.get_required_service::<JobRunner<TData>>();
        let job_worker = JobWorker::new(job_worker_settings, storage.clone(), job_runner.clone());
        let job_worker_handle = job_worker.start();

        log::info!("JobfireManager started");
        Self {
            context,
            job_worker_handle,
        }
    }

    pub async fn stop(self) -> Result<()> {
        log::info!("JobfireManager stopping");
        self.job_worker_handle
            .stop()
            .await
            .map_err(|_| Error::StopFailed)?;

        let job_worker_handle = self.job_worker_handle.clone();

        poll_predicate(
            async move || job_worker_handle.get_state().await == State::Stopped,
            interval(Duration::milliseconds(100).to_std().unwrap()),
        )
        .await;

        log::info!("JobfireManager stopped");
        Ok(())
    }

    pub async fn schedule(
        &self,
        job_impl: impl JobImpl<TData>,
        at: DateTime<Utc>,
    ) -> Result<JobId> {
        let job = Job::from_impl(job_impl).map_err(|_| Error::JobBuildFailed)?;
        let job_id = *job.id();

        self.context
            .get_required_service::<JobScheduler>()
            .schedule(job, at)
            .await?;

        Ok(job_id)
    }

    pub async fn cancel(&self, job_id: &JobId) -> Result<()> {
        self.context
            .get_required_service::<JobScheduler>()
            .cancel(job_id)
            .await?;
        Ok(())
    }

    pub async fn reschedule(
        &self,
        job_id: &JobId,
        new_scheduled_at: DateTime<chrono::Utc>,
    ) -> Result<()> {
        self.context
            .get_required_service::<JobScheduler>()
            .reschedule(job_id, new_scheduled_at)
            .await?;
        Ok(())
    }

    pub fn context(&self) -> &Context<TData> {
        &self.context
    }
}

fn add_default_services<TData: ContextData>(context: &Context<TData>) {
    let services = context.services();
    services.add_service_unchecked(JobWorkerSettings::default());
    services.add_service(JobRunner::new(context.clone()));
    services.add_service(OnSuccessRunner::new(context.clone()));
    services.add_service(OnFailRunner::new(context.clone()));
    services.add_service(JobScheduler::new(services.clone()));
}
