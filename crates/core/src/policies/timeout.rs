use std::{marker::PhantomData, sync::Arc};

use chrono::Duration;
use log::warn;
use tokio::time::timeout;

use crate::domain::{
    job::{
        context::{Context, ContextData},
        error::JobError,
        r#impl::SerializedJobImpl,
        policy::{Policy, PolicyData, PolicyName},
    },
    run::job_actions::RunFn,
};

#[derive(Clone)]
pub struct TimeoutPolicy<TData: ContextData> {
    name: PolicyName,
    timeout: Duration,
    phantom_data: PhantomData<TData>,
}

impl<TData: ContextData> TimeoutPolicy<TData> {
    pub fn new(timeout: Duration) -> Self {
        Self {
            name: PolicyName::new(&format!("jobfire::timeout::{}", timeout)),
            timeout,
            phantom_data: Default::default(),
        }
    }
}

impl<TData: ContextData> Policy<TData> for TimeoutPolicy<TData> {
    fn name(&self) -> PolicyName {
        self.name.clone()
    }

    fn wrap_run(&self, f: RunFn<TData>, _data: PolicyData) -> RunFn<TData> {
        let duration = self.timeout.to_std().unwrap();
        let run: RunFn<TData> = Arc::new(
            move |serialized_job_impl: SerializedJobImpl, job_context: Context<TData>| {
                let f = f.clone();
                Box::pin(async move {
                    let result = timeout(
                        duration,
                        f(serialized_job_impl.clone(), job_context.clone()),
                    )
                    .await;

                    match result {
                        Ok(v) => v,
                        Err(_) => {
                            warn!("job cancelled");
                            Err(JobError::PolicyShortCircuit)
                        }
                    }
                })
            },
        );

        run
    }
}
