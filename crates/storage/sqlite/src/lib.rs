use job::{SqliteJobRepo, pending::SqlitePendingJobRepo, running::SqliteRunningJobRepo};
use jobfire_core::storage::{self, Storage};
use run::{failed::SqliteFailedRunRepo, successful::SqliteSuccessfulRunRepo};
use sqlx::SqlitePool;
use thiserror::Error;

pub mod job;
pub mod run;

#[derive(Error, Debug)]
#[error("failed to initialize storage: ")]
pub struct InitializationFailed(#[from] sqlx::error::Error);

pub type Result<T> = std::result::Result<T, InitializationFailed>;

#[derive(Clone, Debug)]
pub struct SqliteStorageSettings {
    pub(crate) job_table_name: String,
    pub(crate) pending_job_table_name: String,
    pub(crate) running_job_table_name: String,
    pub(crate) successful_run_table_name: String,
    pub(crate) failed_run_table_name: String,
}

impl Default for SqliteStorageSettings {
    fn default() -> Self {
        Self::new(
            "jobfire_job",
            "jobfire_pending_job",
            "jobfire_running_job",
            "jobfire_successful_run",
            "jobfire_failed_run",
        )
    }
}

impl SqliteStorageSettings {
    pub fn new(
        job_table_name: &str,
        pending_job_table_name: &str,
        running_job_table_name: &str,
        successful_run_table_name: &str,
        failed_run_table_name: &str,
    ) -> Self {
        Self {
            job_table_name: job_table_name.to_owned(),
            pending_job_table_name: pending_job_table_name.to_owned(),
            running_job_table_name: running_job_table_name.to_owned(),
            successful_run_table_name: successful_run_table_name.to_owned(),
            failed_run_table_name: failed_run_table_name.to_owned(),
        }
    }
}

pub struct SqliteStorage {
    job_repo: SqliteJobRepo,
    pending_job_repo: SqlitePendingJobRepo,
    running_job_repo: SqliteRunningJobRepo,
    successful_run_repo: SqliteSuccessfulRunRepo,
    failed_run_repo: SqliteFailedRunRepo,
}

impl SqliteStorage {
    pub async fn new(url: &str, settings: SqliteStorageSettings) -> Result<Self> {
        let pool = SqlitePool::connect(url).await?;
        let job_repo = SqliteJobRepo::new(pool.clone(), settings.clone()).await?;
        let pending_job_repo = SqlitePendingJobRepo::new(pool.clone(), settings.clone()).await?;
        let running_job_repo = SqliteRunningJobRepo::new(pool.clone(), settings.clone()).await?;
        let successful_run_repo =
            SqliteSuccessfulRunRepo::new(pool.clone(), settings.clone()).await?;
        let failed_run_repo = SqliteFailedRunRepo::new(pool.clone(), settings.clone()).await?;

        Ok(SqliteStorage {
            job_repo,
            pending_job_repo,
            running_job_repo,
            successful_run_repo,
            failed_run_repo,
        })
    }

    pub async fn new_in_memory() -> Self {
        Self::new(":memory:", Default::default()).await.unwrap()
    }
}

impl From<SqliteStorage> for Storage {
    fn from(value: SqliteStorage) -> Self {
        Storage::new(
            Box::new(value.job_repo),
            Box::new(value.pending_job_repo),
            Box::new(value.running_job_repo),
            Box::new(value.successful_run_repo),
            Box::new(value.failed_run_repo),
        )
    }
}

pub(crate) fn map_sqlx_error(error: sqlx::Error) -> storage::error::Error {
    storage::error::Error::Custom {
        message: error.to_string(),
    }
}
