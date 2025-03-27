use thiserror::Error;

pub mod job;
pub mod run;

#[derive(Error, Debug)]
#[error("failed to initialize storage: ")]
pub struct InitializationFailed(#[from] sqlx::error::Error);

pub type Result<T> = std::result::Result<T, InitializationFailed>;

pub struct SqliteStorageSettings {
    pending_job_repo_table_name: String,
}

impl Default for SqliteStorageSettings {
    fn default() -> Self {
        Self {
            pending_job_repo_table_name: "pending_job".to_owned(),
        }
    }
}

pub struct SqliteStorage {}
