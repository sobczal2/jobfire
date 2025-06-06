use chrono::{DateTime, Utc};
use context::ContextData;
use id::JobId;
use policy::{Policies, Policy, PolicyData};
use r#impl::{JobImpl, SerializedJobImpl};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod context;
pub mod error;
pub mod id;
pub mod r#impl;
pub mod pending;
pub mod policy;
pub mod report;
pub mod running;

#[derive(Error, Debug)]
pub enum Error {
    #[error("building job failed")]
    BuildingJobFailed,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Job information with serialized implementation.
///
/// This structure represents a complete job definition that can be stored
/// in a persistent storage. It contains all the information needed to
/// recreate and execute the job at runtime.
#[derive(Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier for this job.
    ///
    /// Used to reference this job across different components of the system.
    id: JobId,

    /// Timestamp when the job was initially created.
    ///
    /// Used for tracking job lifecycle and potentially for metrics.
    created_at: DateTime<Utc>,

    /// Serialized implementation of the job's functionality.
    ///
    /// Contains the serialized form of the job logic. At runtime,
    /// the actual job functionality is recreated from this serialized data.
    r#impl: SerializedJobImpl,

    /// Policies and policy data invoked for this job
    policies: Policies,
}

impl Job {
    pub fn new(
        id: JobId,
        created_at: DateTime<Utc>,
        r#impl: SerializedJobImpl,
        policies: Policies,
    ) -> Self {
        Self {
            id,
            created_at,
            r#impl,
            policies,
        }
    }

    pub fn id(&self) -> JobId {
        self.id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn policies(&self) -> &Policies {
        &self.policies
    }

    pub fn r#impl(&self) -> &SerializedJobImpl {
        &self.r#impl
    }

    /// Function to create a job from custom job implementation
    pub fn from_impl<TData: ContextData>(
        job_impl: impl JobImpl<TData>,
        now: DateTime<Utc>,
        policies: Vec<Box<dyn Policy<TData>>>,
    ) -> Result<Self> {
        let data = PolicyData::default();
        for policy in policies.iter() {
            policy.init(data.clone());
        }
        Ok(Self::new(
            JobId::default(),
            now,
            SerializedJobImpl::from_job_impl(r#job_impl).map_err(|_| Error::BuildingJobFailed)?,
            Policies::new(policies.iter().map(|p| p.name()).collect::<Vec<_>>(), data),
        ))
    }

    pub fn update_policies(&mut self, policies: Policies) {
        self.policies = policies;
    }
}
