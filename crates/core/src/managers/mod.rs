mod error;

use crate::{domain::job::Context, persistence::Persistence};

pub(crate) struct JobfireManager<T: Context> {
    persistence: Persistence<T>,
}

impl<T: Context> JobfireManager<T> {
    pub fn new(persistence: Persistence<T>) -> error::Result<Self> {
        unimplemented!()
    }
}
