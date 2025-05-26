use std::{marker::PhantomData, sync::Arc};

use log::error;

use crate::{
    domain::job::{
        context::{Context, ContextData},
        data::JobData,
        error::JobError,
        r#impl::SerializedJobImpl,
        policy::{Policy, PolicyName},
    },
    registries::job_actions::{OnFailFn, OnSuccessFn, RunFn},
};

pub struct InstantRetryPolicy<TData: ContextData> {
    max_tries: u32,
    phantom_data: PhantomData<TData>,
}

const MAX_TRIES: &str = "jobfire::instant_retry::MAX_TRIES";
const CURRENT_TRY: &str = "jobfire::instant_retry::CURRENT_TRY";

impl<TData: ContextData> Default for InstantRetryPolicy<TData> {
    fn default() -> Self {
        Self {
            max_tries: 5,
            phantom_data: Default::default(),
        }
    }
}

impl<TData: ContextData> Policy<TData> for InstantRetryPolicy<TData> {
    fn name(&self) -> PolicyName {
        PolicyName::new("jobfire::instant_retry")
    }

    fn init(&self, data: JobData) {
        data.set(MAX_TRIES, self.max_tries).unwrap();
        data.set(CURRENT_TRY, 0u32).unwrap();
    }

    fn wrap_run(&self, f: RunFn<TData>, data: JobData) -> RunFn<TData> {
        let run: RunFn<TData> = Arc::new(
            move |serialized_job_impl: SerializedJobImpl, job_context: Context<TData>| {
                let f = f.clone();
                let data = data.clone();
                Box::pin(async move {
                    let max_tries: u32 = data.get(MAX_TRIES).unwrap().unwrap();
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

    fn wrap_on_fail(&self, f: OnFailFn<TData>, data: JobData) -> OnFailFn<TData> {
        todo!()
    }

    fn wrap_on_success(&self, f: OnSuccessFn<TData>, data: JobData) -> OnSuccessFn<TData> {
        todo!()
    }
}
