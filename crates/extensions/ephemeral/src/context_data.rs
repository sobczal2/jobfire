use std::{ops::Deref, sync::Arc};

use jobfire_core::domain::job::context::{Context, ContextData};

use crate::ephemeral_fn_registry::EphemeralFnRegistry;

pub struct EphemeralContextData<TData: ContextData> {
    inner: Arc<TData>,
    ephemeral_fn_registry: EphemeralFnRegistry<TData>,
}

impl<TData: ContextData> ContextData for EphemeralContextData<TData> {}

impl<TData: ContextData> EphemeralContextData<TData> {
    pub fn registry(&self) -> &EphemeralFnRegistry<TData> {
        &self.ephemeral_fn_registry
    }
}

pub trait IntoInnerContext<TData: ContextData> {
    fn to_inner_context(&self) -> Context<TData>;
}
