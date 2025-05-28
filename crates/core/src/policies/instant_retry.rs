use crate::domain::{
    job::{
        context::{Context, ContextData},
        error::JobError,
        r#impl::SerializedJobImpl,
        policy::{Policy, PolicyData, PolicyName},
    },
    run::job_actions::RunFn,
};
use log::error;
use std::{marker::PhantomData, sync::Arc};

#[derive(Clone)]
pub struct InstantRetryPolicy<TData: ContextData> {
    name: PolicyName,
    max_tries: u32,
    phantom_data: PhantomData<TData>,
}

impl<TData: ContextData> Default for InstantRetryPolicy<TData> {
    fn default() -> Self {
        Self::new(5)
    }
}

impl<TData: ContextData> InstantRetryPolicy<TData> {
    fn max_tries_key(&self) -> String {
        format!("{}::MAX_TRIES", self.name())
    }

    pub fn new(max_tries: u32) -> Self {
        Self {
            name: PolicyName::new(&format!("jobfire::instant_retry::{}", max_tries)),
            max_tries,
            phantom_data: Default::default(),
        }
    }
}

impl<TData: ContextData> Policy<TData> for InstantRetryPolicy<TData> {
    fn name(&self) -> PolicyName {
        self.name.clone()
    }

    fn init(&self, data: PolicyData) {
        data.set(&self.max_tries_key(), self.max_tries).unwrap();
    }

    fn wrap_run(&self, f: RunFn<TData>, data: PolicyData) -> RunFn<TData> {
        let max_tries_key = self.max_tries_key();
        let run: RunFn<TData> = Arc::new(
            move |serialized_job_impl: SerializedJobImpl, job_context: Context<TData>| {
                let f = f.clone();
                let data = data.clone();
                let max_tries_key = max_tries_key.clone();

                Box::pin(async move {
                    let max_tries: u32 = data.get(&max_tries_key).unwrap().unwrap();
                    let mut current_try: u32 = 0;
                    let mut result = Err(JobError::PolicyShortCircuit);
                    while current_try < max_tries {
                        current_try += 1;
                        result = f(serialized_job_impl.clone(), job_context.clone()).await;
                        match result {
                            Ok(_) => break,
                            Err(_) => error!("Retry #{current_try}"),
                        }
                    }
                    result
                })
            },
        );

        run
    }
}
