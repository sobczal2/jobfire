use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::Services;

#[derive(Default)]
pub struct ServicesBuilder {
    inner: Arc<Mutex<ServicesBuilderInner>>,
}

impl Clone for ServicesBuilder {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[derive(Default)]
pub struct ServicesBuilderInner {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ServicesBuilder {
    pub fn add_service<T: 'static + Send + Sync>(&mut self, data: T) {
        self.inner
            .lock()
            .unwrap()
            .map
            .insert(TypeId::of::<T>(), Box::new(data));
    }
}

impl From<ServicesBuilder> for Services {
    fn from(value: ServicesBuilder) -> Self {
        Services::new(value.inner.lock().unwrap().map.drain().collect())
    }
}
