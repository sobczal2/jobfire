pub mod pending;
pub mod running;

use crate::{SqliteStorageSettings, map_sqlx_error};
use async_trait::async_trait;
use jobfire_core::{
    domain::job::{Job, id::JobId},
    storage::{self, job::JobRepo},
};
use sqlx::SqlitePool;

pub struct SqliteJobRepo {
    pool: SqlitePool,
    settings: SqliteStorageSettings,
}

impl SqliteJobRepo {
    pub async fn new(pool: SqlitePool, settings: SqliteStorageSettings) -> crate::Result<Self> {
        Self::init(&pool, &settings).await?;
        Ok(Self { pool, settings })
    }

    async fn init(pool: &SqlitePool, settings: &SqliteStorageSettings) -> crate::Result<()> {
        sqlx::query(&format!(
            "
CREATE TABLE IF NOT EXISTS {} (
    id TEXT PRIMARY KEY,
    created_at INTEGER NOT NULL,
    impl TEXT NOT NULL
)",
            settings.job_table_name,
        ))
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl JobRepo for SqliteJobRepo {
    async fn get(&self, job_id: &JobId) -> storage::error::Result<Option<Job>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: String,
            created_at: i64,
            impl_: String,
        }

        let result = sqlx::query_as::<_, Row>(&format!(
            "SELECT id, created_at, impl AS impl_ FROM {} WHERE id = ?",
            self.settings.job_table_name
        ))
        .bind(job_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        match result {
            Some(row) => {
                let id = row
                    .id
                    .parse()
                    .map_err(|_| storage::error::Error::Internal)?;
                let created_at = chrono::DateTime::from_timestamp_millis(row.created_at)
                    .ok_or(storage::error::Error::Internal)?;
                let r#impl = serde_json::from_str(&row.impl_)
                    .map_err(|_| storage::error::Error::Internal)?;

                Ok(Some(Job::new(id, created_at, r#impl)))
            }
            None => Ok(None),
        }
    }

    async fn add(&self, job: Job) -> storage::error::Result<()> {
        if self.get(job.id()).await?.is_some() {
            return Err(storage::error::Error::AlreadyExists);
        }

        sqlx::query(&format!(
            "INSERT INTO {} (id, created_at, impl) VALUES (?, ?, ?)",
            self.settings.job_table_name
        ))
        .bind(job.id().to_string())
        .bind(job.created_at().timestamp_millis())
        .bind(serde_json::to_string(job.r#impl()).map_err(|_| storage::error::Error::Internal)?)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn delete(&self, job_id: &JobId) -> storage::error::Result<Job> {
        let existing = self.get(job_id).await?;
        if existing.is_none() {
            return Err(storage::error::Error::NotFound);
        }

        sqlx::query(&format!(
            "DELETE FROM {} WHERE id = ?",
            self.settings.job_table_name
        ))
        .bind(job_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(existing.unwrap())
    }
}
