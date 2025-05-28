use async_trait::async_trait;
use chrono::DateTime;
use jobfire_core::{
    domain::run::{failed::FailedRun, id::RunId},
    storage::{self, run::FailedRunRepo},
};
use sqlx::SqlitePool;

use crate::{SqliteStorageSettings, map_sqlx_error};

pub struct SqliteFailedRunRepo {
    pool: SqlitePool,
    settings: SqliteStorageSettings,
}

impl SqliteFailedRunRepo {
    pub async fn new(pool: SqlitePool, settings: SqliteStorageSettings) -> crate::Result<Self> {
        Self::init(&pool, &settings).await?;
        Ok(Self { pool, settings })
    }

    async fn init(pool: &SqlitePool, settings: &SqliteStorageSettings) -> crate::Result<()> {
        sqlx::query(&format!(
            "
CREATE TABLE IF NOT EXISTS {} (
    run_id TEXT NOT NULL PRIMARY KEY,
    job_id TEXT NOT NULL,
    scheduled_at INTEGER NOT NULL,
    finished_at INTEGER NOT NULL,
    error TEXT NOT NULL
)",
            settings.failed_run_table_name,
        ))
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl FailedRunRepo for SqliteFailedRunRepo {
    async fn get(&self, run_id: &RunId) -> storage::error::Result<Option<FailedRun>> {
        #[derive(sqlx::FromRow)]
        struct RGet {
            job_id: String,
            scheduled_at: i64,
            finished_at: i64,
            error: String,
        }

        let result: Option<RGet> = sqlx::query_as(&format!(
            "
SELECT
    job_id,
    scheduled_at,
    finished_at,
    error
FROM {}
WHERE run_id = ?",
            self.settings.failed_run_table_name,
        ))
        .bind(run_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        match result {
            Some(result) => Ok(Some(FailedRun::new(
                *run_id,
                result
                    .job_id
                    .parse()
                    .map_err(|_| storage::error::Error::Internal)?,
                DateTime::from_timestamp_millis(result.scheduled_at)
                    .ok_or(storage::error::Error::Internal)?,
                DateTime::from_timestamp_millis(result.finished_at)
                    .ok_or(storage::error::Error::Internal)?,
                serde_json::from_str(&result.error).map_err(|_| storage::error::Error::Internal)?,
            ))),
            None => Ok(None),
        }
    }

    async fn add(&self, run: FailedRun) -> storage::error::Result<()> {
        let existing_run = self.get(run.run_id()).await?;
        if existing_run.is_some() {
            return Err(storage::error::Error::AlreadyExists);
        }

        sqlx::query(&format!(
            "
INSERT INTO {} (
    job_id,
    run_id,
    scheduled_at,
    finished_at,
    error
)
VALUES
(?, ?, ?, ?, ?)",
            self.settings.failed_run_table_name,
        ))
        .bind(run.job_id().to_string())
        .bind(run.run_id().to_string())
        .bind(run.scheduled_at().timestamp_millis())
        .bind(run.finished_at().timestamp_millis())
        .bind(serde_json::to_string(run.error()).map_err(|_| storage::error::Error::Internal)?)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
