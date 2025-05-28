use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::job::{Job, data::JobData, id::JobId, pending::PendingJob, running::RunningJob};

use super::error::Result;

/// Repository interface for managing `Job` entities.
///
/// This trait defines the core operations for storing, retrieving, and removing
/// jobs from a persistent storage. It encapsulates the storage layer for
/// complete job definitions including their serialized implementations.
#[async_trait]
pub trait JobRepo: Send + Sync + 'static {
    /// Retrieves a job by its job_id.
    ///
    /// # Parameters
    ///
    /// * `job_id` - The job_id of the job to retrieve.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Job>>` - Returns the job if found, None if not found,
    ///   or an error if the retrieval operation failed.
    async fn get(&self, job_id: &JobId) -> Result<Option<Job>>;

    /// Adds a job to the repository.
    ///
    /// # Parameters
    ///
    /// * `job` - The job to add to the repository.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns success if the job was added successfully,
    ///   or an error if the operation failed.
    ///
    /// # Important
    ///
    /// Implementation may fail if a job with the same job_id already exists
    /// in storage.
    async fn add(&self, job: Job) -> Result<()>;

    /// Deletes a job from the repository by its job_id and returns the deleted job.
    ///
    /// # Parameters
    ///
    /// * `job_id` - The job_id of the job to delete.
    ///
    /// # Returns
    ///
    /// * `Result<Job>` - Returns the deleted job on success,
    ///   or an error if the deletion operation failed or the job was not found.
    async fn delete(&self, job_id: &JobId) -> Result<Job>;

    async fn update(&self, job_id: &JobId, data: JobData) -> Result<()>;
}

/// Repository interface for managing `PendingJob` entities.
///
/// This trait defines operations for storing, retrieving, and removing pending jobs
/// from a persistent storage, along with specialized methods for scheduling.
/// Pending jobs represent work that is scheduled but not yet running.
#[async_trait]
pub trait PendingJobRepo: Send + Sync + 'static {
    /// Retrieves a pending job by its job_id.
    ///
    /// # Parameters
    ///
    /// * `job_id` - The job_id of the pending job to retrieve.
    ///
    /// # Returns
    ///
    /// * `Result<Option<PendingJob>>` - Returns the pending job if found, None if not found,
    ///   or an error if the retrieval operation failed.
    async fn get(&self, job_id: &JobId) -> Result<Option<PendingJob>>;

    /// Adds a pending job to the repository.
    ///
    /// # Parameters
    ///
    /// * `job` - The pending job to add to the repository.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns success if the pending job was added successfully,
    ///   or an error if the operation failed.
    ///
    /// # Important
    ///
    /// Implementation may fail if a pending job with the same job_id already exists
    /// in storage.
    async fn add(&self, job: PendingJob) -> Result<()>;

    /// Deletes a pending job from the repository by its identifier and returns the deleted job.
    ///
    /// # Parameters
    ///
    /// * `job_id` - The job_id of the pending job to delete.
    ///
    /// # Returns
    ///
    /// * `Result<PendingJob>` - Returns the deleted pending job on success,
    ///   or an error if the deletion operation failed or the job was not found.
    async fn delete(&self, job_id: &JobId) -> Result<PendingJob>;

    /// Atomically retrieves and removes the next scheduled pending job.
    ///
    /// This method is used for dequeuing work that is ready to be processed.
    ///
    /// # Returns
    ///
    /// * `Result<Option<PendingJob>>` - Returns the next scheduled pending job if available,
    ///   None if no jobs are scheduled, or an error if the operation failed. There is no guarantee
    ///   on order of retrieved jobs.
    async fn pop_scheduled(&self, now: DateTime<Utc>) -> Result<Option<PendingJob>>;
}

/// Repository interface for managing `RunningJob` entities.
///
/// This trait defines operations for storing and removing running jobs
/// from a persistent storage. Running jobs represent work that is
/// currently being executed.
#[async_trait]
pub trait RunningJobRepo: Send + Sync + 'static {
    /// Retrieves a running job by its job_id.
    ///
    /// # Parameters
    ///
    /// * `job_id` - The job_id of the running job to retrieve.
    ///
    /// # Returns
    ///
    /// * `Result<Option<RunningJob>>` - Returns the running job if found, None if not found,
    ///   or an error if the retrieval operation failed.
    async fn get(&self, job_id: &JobId) -> Result<Option<RunningJob>>;

    /// Adds a running job to the repository.
    ///
    /// # Parameters
    ///
    /// * `job` - The running job to add to the repository.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns success if the running job was added successfully,
    ///   or an error if the operation failed.
    ///
    /// # Important
    ///
    /// Implementation may fail if a running job with the same job_id or run_id already exists
    /// in storage.
    async fn add(&self, job: RunningJob) -> Result<()>;

    /// Deletes a running job from the repository by its job_id and returns the deleted job.
    ///
    /// # Parameters
    ///
    /// * `job_id` - The job_id of the running job to delete.
    ///
    /// # Returns
    ///
    /// * `Result<RunningJob>` - Returns the deleted running job on success,
    ///   or an error if the deletion operation failed or the job was not found.
    async fn delete(&self, job_id: &JobId) -> Result<RunningJob>;
}
