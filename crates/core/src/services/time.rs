use std::sync::Arc;

use chrono::{DateTime, Utc};

use super::{
    verify::{ServiceMissing, VerifyService},
    Services,
};

pub trait Clock {
    fn utc_now(&self) -> DateTime<Utc>;
}

impl VerifyService for dyn Clock {
    fn verify(&self, _services: &Services) -> Result<(), ServiceMissing> {
        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn utc_now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Default, Debug, Clone)]
pub struct FixedClock(pub DateTime<Utc>);

impl Clock for FixedClock {
    fn utc_now(&self) -> DateTime<Utc> {
        self.0
    }
}

pub struct AnyClock<'a> {
    inner: Arc<dyn Clock + Send + Sync + 'a>,
}

impl<'a> Clone for AnyClock<'a> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<'a> Clock for AnyClock<'a> {
    fn utc_now(&self) -> DateTime<Utc> {
        self.inner.utc_now()
    }
}
impl<'a> AnyClock<'a> {
    pub fn new(clock: impl Clock + Send + Sync + 'a) -> Self {
        Self {
            inner: Arc::new(clock),
        }
    }
}

impl<'a> VerifyService for AnyClock<'a> {
    fn verify(&self, _services: &Services) -> Result<(), ServiceMissing> {
        Ok(())
    }
}
