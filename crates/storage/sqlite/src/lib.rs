use thiserror::Error;

pub mod job;
pub mod run;

#[derive(Error, Debug)]
#[error("failed to initialize storage: ")]
pub struct InitializationFailed(#[from] sqlx::error::Error);

pub type Result<T> = std::result::Result<T, InitializationFailed>;

pub struct SqliteStorageSettings {
    schema: Option<String>,
    pending_job_repo_table_name: String,
}

impl Default for SqliteStorageSettings {
    fn default() -> Self {
        Self {
            schema: None,
            pending_job_repo_table_name: "pending_job".to_owned(),
        }
    }
}

impl SqliteStorageSettings {
    pub fn pending_job_table_name(&self) -> String {
        match &self.schema {
            Some(schema) => format!("{}.{}", schema, self.pending_job_repo_table_name).clone(),
            None => self.pending_job_repo_table_name.clone(),
        }
    }
}

pub struct SqliteStorage {}
