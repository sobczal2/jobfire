use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// TODO: mutate inside
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct JobData {
    inner: HashMap<String, Value>,
}

impl JobData {}
