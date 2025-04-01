use chrono::{DateTime, Utc};
use jobfire_core::{
    async_trait,
    domain::job::{id::JobId, pending::PendingJob},
    storage::{self, job::PendingJobRepo},
};
use sqlx::SqlitePool;

use crate::{SqliteStorageSettings, map_sqlx_error};

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
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {} (job_id TEXT NOT NULL PRIMARY KEY, scheduled_at INTEGER NOT NULL)",
            settings.pending_job_table_name,
        ))
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl PendingJobRepo for SqlitePendingJobRepo {
    async fn get(&self, job_id: &JobId) -> storage::error::Result<Option<PendingJob>> {
        #[derive(sqlx::FromRow)]
        struct RGet {
            scheduled_at: i64,
        }

        let result: Option<RGet> = sqlx::query_as(&format!(
            "SELECT scheduled_at FROM {} WHERE job_id = $1",
            self.settings.pending_job_table_name,
        ))
        .bind(job_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        match result {
            Some(result) => Ok(Some(PendingJob::new(
                *job_id,
                DateTime::from_timestamp_millis(result.scheduled_at)
                    .ok_or(storage::error::Error::Internal)?,
            ))),
            None => Ok(None),
        }
    }

    async fn add(&self, job: PendingJob) -> storage::error::Result<()> {
        let existing_job = self.get(job.job_id()).await?;
        if existing_job.is_some() {
            return Err(storage::error::Error::AlreadyExists);
        }

        sqlx::query(&format!(
            "INSERT INTO {} (job_id, scheduled_at) VALUES ($1, $2)",
            self.settings.pending_job_table_name,
        ))
        .bind(job.job_id().to_string())
        .bind(job.scheduled_at().timestamp_millis())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete(&self, job_id: &JobId) -> storage::error::Result<PendingJob> {
        let existing_job = self.get(job_id).await?;
        if existing_job.is_none() {
            return Err(storage::error::Error::NotFound);
        }

        sqlx::query(&format!(
            "DELETE FROM {} WHERE job_id == $1",
            self.settings.pending_job_table_name
        ))
        .bind(job_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(existing_job.unwrap())
    }

    async fn pop_scheduled(
        &self,
        now: DateTime<Utc>,
    ) -> storage::error::Result<Option<PendingJob>> {
        #[derive(sqlx::FromRow)]
        struct RGet {
            job_id: String,
            scheduled_at: i64,
        }

        let timestamp = now.timestamp_millis();

        let existing_job: Option<RGet> = sqlx::query_as(&format!(
            "SELECT job_id, scheduled_at FROM {} WHERE scheduled_at < $1 ORDER BY scheduled_at ASC LIMIT 1",
            self.settings.pending_job_table_name
        ))
        .bind(timestamp)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let existing_job = match existing_job {
            Some(job) => PendingJob::new(
                job.job_id
                    .parse()
                    .map_err(|_| storage::error::Error::Internal)?,
                DateTime::from_timestamp_millis(job.scheduled_at)
                    .ok_or(storage::error::Error::Internal)?,
            ),
            None => return Ok(None),
        };

        self.delete(existing_job.job_id()).await?;

        Ok(Some(existing_job))
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

    #[tokio::test]
    async fn test_add_job() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqlitePendingJobRepo::new(pool, settings).await.unwrap();

        let job = PendingJob::new(
            JobId::default(),
            DateTime::from_timestamp_millis(1).unwrap(),
        );

        repo.add(job.clone()).await.unwrap();

        let retrieved = repo.get(job.job_id()).await.unwrap().unwrap();
        assert_eq!(retrieved.job_id(), job.job_id());
        assert_eq!(retrieved.scheduled_at(), job.scheduled_at());
    }

    #[tokio::test]
    async fn test_add_duplicate_job() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqlitePendingJobRepo::new(pool, settings).await.unwrap();

        let job = PendingJob::new(
            JobId::default(),
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
        let repo = SqlitePendingJobRepo::new(pool, settings).await.unwrap();

        let job = PendingJob::new(
            JobId::default(),
            DateTime::from_timestamp_millis(1).unwrap(),
        );

        repo.add(job.clone()).await.unwrap();

        let retrieved = repo.get(job.job_id()).await.unwrap().unwrap();
        assert_eq!(retrieved.job_id(), job.job_id());
        assert_eq!(retrieved.scheduled_at(), job.scheduled_at());
    }

    #[tokio::test]
    async fn test_get_nonexistent_job() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqlitePendingJobRepo::new(pool, settings).await.unwrap();

        let job_id = JobId::default();

        let result = repo.get(&job_id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_existing_job() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqlitePendingJobRepo::new(pool, settings).await.unwrap();

        let job = PendingJob::new(
            JobId::default(),
            DateTime::from_timestamp_millis(1).unwrap(),
        );

        repo.add(job.clone()).await.unwrap();

        let deleted = repo.delete(job.job_id()).await.unwrap();
        assert_eq!(deleted.job_id(), job.job_id());
        assert_eq!(deleted.scheduled_at(), job.scheduled_at());

        let result = repo.get(job.job_id()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_job() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqlitePendingJobRepo::new(pool, settings).await.unwrap();

        let job_id = JobId::default();

        let result = repo.delete(&job_id).await;
        assert!(matches!(result, Err(storage::error::Error::NotFound)));
    }

    #[tokio::test]
    async fn test_pop_scheduled_no_jobs() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqlitePendingJobRepo::new(pool, settings).await.unwrap();

        let now = Utc::now();
        let result = repo.pop_scheduled(now).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_pop_scheduled_before_time() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqlitePendingJobRepo::new(pool, settings).await.unwrap();

        let job = PendingJob::new(
            JobId::default(),
            DateTime::from_timestamp_millis(100).unwrap(),
        );

        repo.add(job.clone()).await.unwrap();

        let before = DateTime::from_timestamp_millis(50).unwrap();
        let result = repo.pop_scheduled(before).await.unwrap();
        assert!(result.is_none());

        let retrieved = repo.get(job.job_id()).await.unwrap();
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_pop_scheduled_after_time() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqlitePendingJobRepo::new(pool, settings).await.unwrap();

        let job = PendingJob::new(
            JobId::default(),
            DateTime::from_timestamp_millis(100).unwrap(),
        );

        repo.add(job.clone()).await.unwrap();

        let after = DateTime::from_timestamp_millis(150).unwrap();
        let popped = repo.pop_scheduled(after).await.unwrap().unwrap();
        assert_eq!(popped.job_id(), job.job_id());
        assert_eq!(popped.scheduled_at(), job.scheduled_at());

        let retrieved = repo.get(job.job_id()).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_pop_scheduled_multiple_jobs() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let settings = SqliteStorageSettings::default();
        let repo = SqlitePendingJobRepo::new(pool, settings).await.unwrap();

        let job_id1 = JobId::default();
        let job_id2 = JobId::default();
        assert_ne!(job_id1, job_id2, "Test requires different job IDs");

        let job1 = PendingJob::new(job_id1, DateTime::from_timestamp_millis(100).unwrap());

        let job2 = PendingJob::new(job_id2, DateTime::from_timestamp_millis(200).unwrap());

        repo.add(job1.clone()).await.unwrap();
        repo.add(job2.clone()).await.unwrap();

        let after = DateTime::from_timestamp_millis(300).unwrap();

        let popped1 = repo.pop_scheduled(after).await.unwrap().unwrap();
        assert_eq!(popped1.job_id(), job1.job_id());
        assert_eq!(popped1.scheduled_at(), job1.scheduled_at());

        let popped2 = repo.pop_scheduled(after).await.unwrap().unwrap();
        assert_eq!(popped2.job_id(), job2.job_id());
        assert_eq!(popped2.scheduled_at(), job2.scheduled_at());

        let popped3 = repo.pop_scheduled(after).await.unwrap();
        assert!(popped3.is_none());
    }
}
