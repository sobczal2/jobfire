use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{
    domain::job::{self, context::ContextData, id::JobId, pending::PendingJob},
    services::{
        Services,
        verify::{ServiceMissing, VerifyService},
    },
    storage::{self, Storage},
    verify_services,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("storage error: {0}")]
    Storage(#[from] storage::error::Error),
    #[error("job not found")]
    JobNotFound,
    #[error("already scheduled")]
    AlreadyScheduled,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct JobScheduler<TData: ContextData> {
    services: Services<TData>,
}

impl<TData: ContextData> Clone for JobScheduler<TData> {
    fn clone(&self) -> Self {
        Self {
            services: self.services.clone(),
        }
    }
}

impl<TData: ContextData> JobScheduler<TData> {
    pub fn new(services: Services<TData>) -> Self {
        Self { services }
    }
}

impl<TData: ContextData> VerifyService<TData> for JobScheduler<TData> {
    fn verify(&self, services: &Services<TData>) -> std::result::Result<(), ServiceMissing> {
        verify_services!(services, Storage);
        Ok(())
    }
}

impl<TData: ContextData> JobScheduler<TData> {
    pub async fn schedule(&self, job: job::Job, scheduled_at: DateTime<Utc>) -> Result<()> {
        let storage = self.services.get_required_service::<Storage>();

        let pending_job = PendingJob::new(*job.id(), scheduled_at);
        let existing_job = storage.job_repo().get(job.id()).await?;
        if existing_job.is_some() {
            return Err(Error::AlreadyScheduled);
        }

        storage.job_repo().add(job).await?;
        storage.pending_job_repo().add(pending_job).await?;
        Ok(())
    }

    pub async fn cancel(&self, job_id: &JobId) -> Result<()> {
        let storage = self.services.get_required_service::<Storage>();

        // TODO add cancel queue
        match storage.pending_job_repo().delete(job_id).await {
            Ok(_) => Ok(()),
            Err(error) => match error {
                storage::error::Error::NotFound => Err(Error::JobNotFound),
                _ => Err(Error::Storage(error)),
            },
        }
    }

    pub async fn reschedule(&self, job_id: &JobId, new_scheduled_at: DateTime<Utc>) -> Result<()> {
        let storage = self.services.get_required_service::<Storage>();

        let scheduled_job = storage.pending_job_repo().get(job_id).await?;
        if scheduled_job.is_none() {
            return Err(Error::JobNotFound);
        }
        let mut scheduled_job = scheduled_job.unwrap();
        scheduled_job.reschedule(new_scheduled_at);
        storage.pending_job_repo().delete(job_id).await?;
        storage.pending_job_repo().add(scheduled_job).await?;
        Ok(())
    }
}
