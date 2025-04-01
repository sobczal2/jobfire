use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Unique identifier for a job run.
#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct RunId(Uuid);

impl RunId {
    pub fn new(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn value(&self) -> &Uuid {
        &self.0
    }
}

impl Default for RunId {
    fn default() -> Self {
        Self::new(Uuid::now_v7())
    }
}

impl Display for RunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
#[error("failed to parse RunId")]
pub struct RunIdParseError;

impl FromStr for RunId {
    type Err = RunIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Uuid>() {
            Ok(uuid) => Ok(Self::new(uuid)),
            Err(_) => Err(RunIdParseError),
        }
    }
}
