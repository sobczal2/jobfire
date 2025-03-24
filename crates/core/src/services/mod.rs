pub mod verify;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use verify::{ServiceMissing, VerifyService};

pub struct Services {
    inner: Arc<RwLock<ServicesInner>>,
}

impl Clone for Services {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for Services {
    fn default() -> Self {
        Self::new(Default::default(), Default::default())
    }
}

struct ServicesInner {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    verify_services: Vec<Box<dyn VerifyService + Send + Sync>>,
}

impl Services {
    pub fn new(
        map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
        verify_services: Vec<Box<dyn VerifyService + Send + Sync>>,
    ) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ServicesInner {
                map,
                verify_services,
            })),
        }
    }

    pub fn get_service<T: Clone + 'static>(&self) -> Option<T> {
        self.inner
            .read()
            .unwrap()
            .map
            .get(&TypeId::of::<T>())
            .and_then(|data| data.downcast_ref::<T>())
            .cloned()
    }

    pub fn get_required_service<T: Clone + 'static>(&self) -> T {
        self.inner
            .read()
            .unwrap()
            .map
            .get(&TypeId::of::<T>())
            .expect("failed to get required service")
            .downcast_ref::<T>()
            .expect("failed to downcast_ref")
            .clone()
    }

    pub fn is_registered<T: Clone + 'static>(&self) -> bool {
        self.get_service::<T>().is_some()
    }

    pub fn add_service_unchecked<T: Clone + Send + Sync + 'static>(&self, service: T) {
        self.inner
            .write()
            .unwrap()
            .map
            .insert(TypeId::of::<T>(), Box::new(service));
    }

    pub fn add_service<T: VerifyService + Clone + Send + Sync + 'static>(&self, service: T) {
        self.add_service_unchecked(service.clone());
        self.verify_service_on_build(service.clone());
    }

    pub fn verify_service_on_build<T: VerifyService + Clone + Send + Sync + 'static>(
        &self,
        verify_service: T,
    ) {
        self.inner
            .write()
            .unwrap()
            .verify_services
            .push(Box::new(verify_service));
    }

    pub fn verify(&self) -> Result<(), ServiceMissing> {
        for vs in self.inner.read().unwrap().verify_services.iter() {
            vs.verify(&self)?;
        }
        Ok(())
    }
}
