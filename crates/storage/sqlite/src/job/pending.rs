use chrono::DateTime;
use jobfire_core::{
    async_trait,
    domain::job::{id::JobId, pending::PendingJob},
    storage::{self, job::PendingJobRepo},
};
use sqlx::{SqlitePool, prelude::*};

use crate::SqliteStorageSettings;

pub struct SqlitePendingJobRepo {
    pool: SqlitePool,
    settings: SqliteStorageSettings,
}

impl SqlitePendingJobRepo {
    pub async fn new(pool: SqlitePool, settings: SqliteStorageSettings) -> crate::Result<Self> {
        Self::init(&pool, &settings).await?;
        Ok(Self { pool, settings })
    }

    async fn init(pool: &SqlitePool, settings: &SqliteStorageSettings) -> crate::Result<()> {
        sqlx::query(
            format!(
                "CREATE TABLE IF NOT EXISTS {} (job_id TEXT PRIMARY KEY, scheduled_at INTEGER)",
                &settings.pending_job_repo_table_name,
            )
            .as_str(),
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    fn map_sqlx_error(error: sqlx::Error) -> storage::error::Error {
        storage::error::Error::Custom {
            message: error.to_string(),
        }
    }
}

#[async_trait]
impl PendingJobRepo for SqlitePendingJobRepo {
    async fn get(&self, job_id: &JobId) -> storage::error::Result<Option<PendingJob>> {
        #[derive(sqlx::FromRow)]
        struct RGet {
            scheduled_at: i64,
        }

        let result: Option<RGet> = sqlx::query_as("SELECT scheduled_at FROM $1 WHERE job_id = $2")
            .bind(&self.settings.pending_job_repo_table_name)
            .bind(job_id.value().to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(Self::map_sqlx_error)?;

        match result {
            Some(result) => Ok(Some(PendingJob::new(
                *job_id,
                DateTime::from_timestamp_nanos(result.scheduled_at),
            ))),
            None => Ok(None),
        }
    }

    async fn add(&self, job: PendingJob) -> storage::error::Result<()> {
        todo!()
    }

    async fn delete(&self, job_id: &JobId) -> storage::error::Result<PendingJob> {
        todo!()
    }

    async fn pop_scheduled(&self) -> storage::error::Result<Option<PendingJob>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();

        SqlitePendingJobRepo::init(&pool, &settings).await.unwrap();
    }
}
