pub mod verify;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use verify::{ServiceMissing, VerifyService};

use crate::domain::job::context::ContextData;

pub struct Services<TData: ContextData> {
    inner: Arc<RwLock<ServicesInner<TData>>>,
}

impl<TData: ContextData> Clone for Services<TData> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TData: ContextData> Default for Services<TData> {
    fn default() -> Self {
        Self::new(Default::default(), Default::default())
    }
}

struct ServicesInner<TData: ContextData> {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    verify_services: Vec<Box<dyn VerifyService<TData> + Send + Sync>>,
    phantom_data: PhantomData<TData>,
}

impl<TData: ContextData> Services<TData> {
    pub fn new(
        map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
        verify_services: Vec<Box<dyn VerifyService<TData> + Send + Sync>>,
    ) -> Self {
        Self {
            inner: Arc::new(RwLock::new(ServicesInner {
                map,
                verify_services,
                phantom_data: Default::default(),
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

    pub fn add_service_unchecked<T: Clone + Send + Sync + 'static>(&self, service: T) -> Self {
        self.inner
            .write()
            .unwrap()
            .map
            .insert(TypeId::of::<T>(), Box::new(service));

        self.clone()
    }

    pub fn add_service<T: VerifyService<TData> + Clone + Send + Sync + 'static>(
        &self,
        service: T,
    ) -> Self {
        self.add_service_unchecked(service.clone());
        self.verify_service_on_build(service.clone());
        self.clone()
    }

    pub fn verify_service_on_build<T: VerifyService<TData> + Clone + Send + Sync + 'static>(
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
            vs.verify(self)?;
        }
        Ok(())
    }
}
