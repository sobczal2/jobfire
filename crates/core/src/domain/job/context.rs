use std::sync::Arc;

use crate::services::Services;

/// Marker trait for context data accessible from jobs.
/// Types implementing this must be `Send` + `Sync` + `'static`.
pub trait ContextData: Send + Sync + 'static {}

/// Context for every job execution.
/// Provides access to data and allows scheduling new jobs.
pub struct Context<TData: ContextData> {
    data: Arc<TData>,
    services: Services,
}

impl<TData: ContextData> Clone for Context<TData> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            services: self.services.clone(),
        }
    }
}

impl<TData: ContextData> Context<TData> {
    pub fn new(data: impl Into<TData>, services: Services) -> Self {
        Self {
            data: Arc::new(data.into()),
            services,
        }
    }

    pub fn data(&self) -> Arc<TData> {
        self.data.clone()
    }

    pub fn services(&self) -> &Services {
        &self.services
    }

    pub fn get_service<T: Clone + 'static>(&self) -> Option<T> {
        self.services.get_service()
    }

    pub fn get_required_service<T: Clone + 'static>(&self) -> T {
        self.services.get_required_service()
    }
}

pub struct EmptyContextData;

impl ContextData for EmptyContextData {}
