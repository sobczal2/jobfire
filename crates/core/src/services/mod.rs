pub mod builder;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

pub struct Services {
    inner: Arc<ServicesInner>,
}

impl Clone for Services {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

struct ServicesInner {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Services {
    pub fn new(map: HashMap<TypeId, Box<dyn Any + Send + Sync>>) -> Self {
        Self {
            inner: Arc::new(ServicesInner { map }),
        }
    }

    pub fn get_service<T: 'static>(&self) -> Option<&T> {
        self.inner
            .map
            .get(&TypeId::of::<T>())
            .and_then(|data| data.downcast_ref::<T>())
    }
}
