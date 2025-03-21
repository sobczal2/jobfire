use chrono::{DateTime, Utc};
use getset::Getters;
use id::JobId;
use r#impl::SerializedJobImpl;
use serde::{Deserialize, Serialize};

pub mod context;
pub mod error;
pub mod failed;
pub mod id;
pub mod r#impl;
pub mod pending;
pub mod report;
pub mod running;
pub mod scheduler;
pub mod successful;

#[derive(Clone, Getters, Serialize, Deserialize)]
#[getset(get = "pub")]
pub struct Job {
    id: JobId,
    created_at: DateTime<Utc>,
    r#impl: SerializedJobImpl,
}
