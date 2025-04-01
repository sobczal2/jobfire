use jobfire_core::storage;
use thiserror::Error;

pub mod job;
pub mod run;

#[derive(Error, Debug)]
#[error("failed to initialize storage: ")]
pub struct InitializationFailed(#[from] sqlx::error::Error);

pub type Result<T> = std::result::Result<T, InitializationFailed>;

pub struct SqliteStorageSettings {
    pub(crate) pending_job_table_name: String,
    pub(crate) running_job_table_name: String,
}

impl Default for SqliteStorageSettings {
    fn default() -> Self {
        Self {
            pending_job_table_name: "pending_job".to_owned(),
            running_job_table_name: "running_job".to_owned(),
        }
    }
}

impl SqliteStorageSettings {
    pub fn new(
        pending_job_table_name: impl Into<String>,
        running_job_table_name: impl Into<String>,
    ) -> Self {
        Self {
            pending_job_table_name: pending_job_table_name.into(),
            running_job_table_name: running_job_table_name.into(),
        }
    }
}

pub struct SqliteStorage {}

pub(crate) fn map_sqlx_error(error: sqlx::Error) -> storage::error::Error {
    storage::error::Error::Custom {
        message: error.to_string(),
    }
}
