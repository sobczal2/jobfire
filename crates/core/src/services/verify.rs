use thiserror::Error;

use super::Services;

#[derive(Error, Debug)]
#[error("service missing: {0}")]
pub struct ServiceMissing(String);

impl ServiceMissing {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self(service_name.into())
    }
}

pub trait VerifyService {
    fn verify(&self, services: &Services) -> Result<(), ServiceMissing>;
}

#[macro_export]
macro_rules! verify_services {
    ($services:expr, $service_name:ty) => {
        if !$services.is_registered::<$service_name>() {
            return Err(ServiceMissing::new(stringify!($service_name)));
        }
    };

    ($services:expr, $service_name:ty, $($rest:ty),+) => {
        verify_services!($services, $service_name);
        verify_services!($services, $($rest),+);
    };
}
