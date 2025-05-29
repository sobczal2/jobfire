use chrono::DateTime;
use jobfire_core::{
    async_trait,
    domain::job::{id::JobId, running::RunningJob},
    storage::{self, job::RunningJobRepo},
};
use sqlx::SqlitePool;

use crate::{SqliteStorageSettings, map_sqlx_error};

pub struct SqliteRunningJobRepo {
    pool: SqlitePool,
    settings: SqliteStorageSettings,
}

impl SqliteRunningJobRepo {
    pub async fn new(pool: SqlitePool, settings: SqliteStorageSettings) -> crate::Result<Self> {
        Self::init(&pool, &settings).await?;
        Ok(Self { pool, settings })
    }

    async fn init(pool: &SqlitePool, settings: &SqliteStorageSettings) -> crate::Result<()> {
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {} (job_id TEXT NOT NULL PRIMARY KEY, run_id TEXT NOT NULL, started_at INTEGER NOT NULL)",
            settings.running_job_table_name,
        ))
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl RunningJobRepo for SqliteRunningJobRepo {
    async fn get(&self, job_id: &JobId) -> storage::error::Result<Option<RunningJob>> {
        #[derive(sqlx::FromRow)]
        struct RGet {
            run_id: String,
            started_at: i64,
        }

        let result: Option<RGet> = sqlx::query_as(&format!(
            "SELECT run_id, started_at FROM {} WHERE job_id = ?",
            self.settings.running_job_table_name,
        ))
        .bind(job_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        match result {
            Some(result) => Ok(Some(RunningJob::new(
                *job_id,
                result
                    .run_id
                    .parse()
                    .map_err(|_| storage::error::Error::Internal)?,
                DateTime::from_timestamp_millis(result.started_at)
                    .ok_or(storage::error::Error::Internal)?,
            ))),
            None => Ok(None),
        }
    }

    async fn add(&self, job: RunningJob) -> storage::error::Result<()> {
        let existing_job = self.get(&job.job_id()).await?;
        if existing_job.is_some() {
            return Err(storage::error::Error::AlreadyExists);
        }

        sqlx::query(&format!(
            "INSERT INTO {} (job_id, run_id, started_at) VALUES (?, ?, ?)",
            self.settings.running_job_table_name,
        ))
        .bind(job.job_id().to_string())
        .bind(job.run_id().to_string())
        .bind(job.started_at().timestamp_millis())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete(&self, job_id: &JobId) -> storage::error::Result<RunningJob> {
        let existing_job = self.get(job_id).await?;
        if existing_job.is_none() {
            return Err(storage::error::Error::NotFound);
        }

        sqlx::query(&format!(
            "DELETE FROM {} WHERE job_id = ?",
            self.settings.running_job_table_name
        ))
        .bind(job_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(existing_job.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use jobfire_core::domain::run::id::RunId;

    use super::*;

    #[tokio::test]
    async fn test_init() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();

        SqliteRunningJobRepo::init(&pool, &settings).await.unwrap();
    }

    #[tokio::test]
    async fn test_add_job() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqliteRunningJobRepo::new(pool, settings).await.unwrap();

        let job = RunningJob::new(
            JobId::default(),
            RunId::default(),
            DateTime::from_timestamp_millis(1).unwrap(),
        );

        repo.add(job.clone()).await.unwrap();

        let retrieved = repo.get(&job.job_id()).await.unwrap().unwrap();
        assert_eq!(retrieved.job_id(), job.job_id());
        assert_eq!(retrieved.run_id(), job.run_id());
        assert_eq!(retrieved.started_at(), job.started_at());
    }

    #[tokio::test]
    async fn test_add_duplicate_job() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqliteRunningJobRepo::new(pool, settings).await.unwrap();

        let job = RunningJob::new(
            JobId::default(),
            RunId::default(),
            DateTime::from_timestamp_millis(1).unwrap(),
        );

        repo.add(job.clone()).await.unwrap();

        let result = repo.add(job.clone()).await;
        assert!(matches!(result, Err(storage::error::Error::AlreadyExists)));
    }

    #[tokio::test]
    async fn test_get_existing_job() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqliteRunningJobRepo::new(pool, settings).await.unwrap();

        let job = RunningJob::new(
            JobId::default(),
            RunId::default(),
            DateTime::from_timestamp_millis(1).unwrap(),
        );

        repo.add(job.clone()).await.unwrap();

        let retrieved = repo.get(&job.job_id()).await.unwrap().unwrap();
        assert_eq!(retrieved.job_id(), job.job_id());
        assert_eq!(retrieved.run_id(), job.run_id());
        assert_eq!(retrieved.started_at(), job.started_at());
    }

    #[tokio::test]
    async fn test_get_nonexistent_job() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqliteRunningJobRepo::new(pool, settings).await.unwrap();

        let job_id = JobId::default();

        let result = repo.get(&job_id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_existing_job() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqliteRunningJobRepo::new(pool, settings).await.unwrap();

        let job = RunningJob::new(
            JobId::default(),
            RunId::default(),
            DateTime::from_timestamp_millis(1).unwrap(),
        );

        repo.add(job.clone()).await.unwrap();

        let deleted = repo.delete(&job.job_id()).await.unwrap();
        assert_eq!(deleted.job_id(), job.job_id());
        assert_eq!(deleted.run_id(), job.run_id());
        assert_eq!(deleted.started_at(), job.started_at());

        let result = repo.get(&job.job_id()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_job() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqliteRunningJobRepo::new(pool, settings).await.unwrap();

        let job_id = JobId::default();

        let result = repo.delete(&job_id).await;
        assert!(matches!(result, Err(storage::error::Error::NotFound)));
    }
}
