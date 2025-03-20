use chrono::{DateTime, Utc};
use getset::Getters;
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};
use thiserror::Error;

use super::{
    context::JobContextData,
    id::JobId,
    r#impl::{JobImpl, JobImplName},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("serialization failed")]
    SerializationFailed,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Getters, Serialize, Deserialize, Hash)]
#[getset(get = "pub")]
pub struct PendingJob {
    id: JobId,
    created_at: DateTime<Utc>,
    scheduled_at: DateTime<Utc>,
    r#impl: Value,
    impl_name: JobImplName,
}

impl PendingJob {
    pub(crate) fn new<TData: JobContextData>(
        id: JobId,
        created_at: DateTime<Utc>,
        scheduled_at: DateTime<Utc>,
        r#impl: impl JobImpl<TData>,
    ) -> Result<Self> {
        let impl_name = r#impl.name_dyn();
        Ok(Self {
            id,
            created_at,
            scheduled_at,
            r#impl: to_value(r#impl).map_err(|_| Error::SerializationFailed)?,
            impl_name,
        })
    }

    pub fn new_at<TJobContext: JobContextData>(
        scheduled_at: DateTime<Utc>,
        r#impl: impl JobImpl<TJobContext>,
    ) -> Result<Self> {
        Self::new(JobId::new(), Utc::now(), scheduled_at, r#impl)
    }

    pub fn reschedule(&mut self, new_scheduled_at: DateTime<Utc>) {
        self.scheduled_at = new_scheduled_at;
    }
}
