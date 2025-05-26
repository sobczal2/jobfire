use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, to_value, Value};
use thiserror::Error;

#[derive(Default)]
pub struct JobData {
    inner: Arc<RwLock<JobDataInner>>,
}

impl Serialize for JobData {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = self.inner.read().map_err(serde::ser::Error::custom)?;
        data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for JobData {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = JobDataInner::deserialize(deserializer)?;
        Ok(JobData {
            inner: Arc::new(RwLock::new(value)),
        })
    }
}

impl Clone for JobData {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct JobDataInner {
    values: HashMap<String, Value>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to cast")]
    CastError,
}

pub type Result<T> = std::result::Result<T, Error>;

impl JobData {
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        match self.inner.read().unwrap().values.get(key) {
            Some(v) => Ok(Some(
                from_value::<T>(v.clone()).map_err(|_| Error::CastError)?,
            )),
            None => Ok(None),
        }
    }

    pub fn set<T: Serialize>(&self, key: &str, value: T) -> Result<()> {
        self.inner.write().unwrap().values.insert(
            key.to_owned(),
            to_value(value).map_err(|_| Error::CastError)?,
        );

        Ok(())
    }
}
