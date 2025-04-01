use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Unique identifier for a job.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct JobId(Uuid);

impl JobId {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn value(&self) -> &Uuid {
        &self.0
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new(Uuid::now_v7())
    }
}

impl Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
#[error("failed to parse JobId")]
pub struct JobIdParseError;

impl FromStr for JobId {
    type Err = JobIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Uuid>() {
            Ok(uuid) => Ok(Self::new(uuid)),
            Err(_) => Err(JobIdParseError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        let uuid_str = "e2cd173e-0880-4781-9a4d-1710d14ce7f4";
        let uuid = uuid::uuid!("e2cd173e-0880-4781-9a4d-1710d14ce7f4");

        assert_eq!(uuid_str, JobId::new(uuid).to_string());
    }

    #[test]
    fn test_from_str() {
        let uuid_str = "e2cd173e-0880-4781-9a4d-1710d14ce7f4";
        let uuid = uuid::uuid!("e2cd173e-0880-4781-9a4d-1710d14ce7f4");
        let job_id = JobId::from_str(uuid_str).unwrap();

        assert_eq!(uuid, job_id.0);
    }
}
