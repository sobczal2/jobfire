use crate::{
    domain::job::context::{JobContext, JobContextData},
    storage::Storage,
};

#[derive(Default)]
pub struct JobfireManagerBuilder<TData: JobContextData> {
    storage: Option<Storage>,
    context: Option<JobContext<TData>>,
}
