use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Report {}

impl Report {
    pub fn new() -> Self {
        Report {}
    }
}
