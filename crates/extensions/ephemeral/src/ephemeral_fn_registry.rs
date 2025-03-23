#![allow(type_alias_bounds)]

use std::{collections::HashMap, sync::Arc};

use jobfire_core::{
    domain::job::context::ContextData,
    registries::job_actions::{OnFailFn, OnSuccessFn, RunFn},
};
use tokio::sync::RwLock;

use crate::job::{EphemeralJobId, SerializedEphemeralJobImpl};

pub type EphemeralRunFn<TData: ContextData> = RunFn<TData>;

pub type EphemeralOnSuccessFn<TData: ContextData> = OnSuccessFn<TData>;

pub type EphemeralOnFailFn<TData: ContextData> = OnFailFn<TData>;

pub struct EphemeralActions<TData: ContextData> {
    run: EphemeralRunFn<TData>,
    on_success: EphemeralOnSuccessFn<TData>,
    on_fail: EphemeralOnFailFn<TData>,
}

pub struct EphemeralFnRegistry<TData: ContextData> {
    inner: Arc<RwLock<EphemeralFnRegistryInner<TData>>>,
}

pub struct EphemeralFnRegistryInner<TData: ContextData> {
    ephemeral_actions_map: HashMap<EphemeralJobId, EphemeralActions<TData>>,
}

impl<TData: ContextData> EphemeralFnRegistry<TData> {
    pub fn new(ephemeral_actions_map: HashMap<EphemeralJobId, EphemeralActions<TData>>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(EphemeralFnRegistryInner {
                ephemeral_actions_map,
            })),
        }
    }
    
    pub fn run(&self, r#impl: SerializedEphemeralJobImpl)
}
