use async_trait::async_trait;
use chrono::DateTime;
use jobfire_core::{
    domain::run::{id::RunId, successful::SuccessfulRun},
    storage::{self, run::SuccessfulRunRepo},
};
use sqlx::SqlitePool;

use crate::{SqliteStorageSettings, map_sqlx_error};

pub struct SqliteSuccessfulRunRepo {
    pool: SqlitePool,
    settings: SqliteStorageSettings,
}

impl SqliteSuccessfulRunRepo {
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
    report TEXT NOT NULL
)
",
            settings.successful_run_table_name,
        ))
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl SuccessfulRunRepo for SqliteSuccessfulRunRepo {
    async fn get(&self, run_id: &RunId) -> storage::error::Result<Option<SuccessfulRun>> {
        #[derive(sqlx::FromRow)]
        struct RGet {
            job_id: String,
            scheduled_at: i64,
            finished_at: i64,
            report: String,
        }

        let result: Option<RGet> = sqlx::query_as(&format!(
            "
SELECT
    job_id,
    scheduled_at,
    finished_at,
    report
FROM {}
WHERE run_id = ?
",
            self.settings.successful_run_table_name,
        ))
        .bind(run_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        match result {
            Some(row) => Ok(Some(SuccessfulRun::new(
                *run_id,
                row.job_id
                    .parse()
                    .map_err(|_| storage::error::Error::Internal)?,
                DateTime::from_timestamp_millis(row.scheduled_at)
                    .ok_or(storage::error::Error::Internal)?,
                DateTime::from_timestamp_millis(row.finished_at)
                    .ok_or(storage::error::Error::Internal)?,
                serde_json::from_str(&row.report).map_err(|_| storage::error::Error::Internal)?,
            ))),
            None => Ok(None),
        }
    }

    async fn add(&self, run: SuccessfulRun) -> storage::error::Result<()> {
        let existing = self.get(&run.run_id()).await?;
        if existing.is_some() {
            return Err(storage::error::Error::AlreadyExists);
        }

        sqlx::query(&format!(
            "
INSERT INTO {} (
    run_id,
    job_id,
    scheduled_at,
    finished_at,
    report
)
VALUES (?, ?, ?, ?, ?)
",
            self.settings.successful_run_table_name,
        ))
        .bind(run.run_id().to_string())
        .bind(run.job_id().to_string())
        .bind(run.scheduled_at().timestamp_millis())
        .bind(run.finished_at().timestamp_millis())
        .bind(serde_json::to_string(run.report()).map_err(|_| storage::error::Error::Internal)?)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
