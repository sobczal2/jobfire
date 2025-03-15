use std::fmt::Display;

use quick_macros::FullCtor;
use thiserror::Error;

#[derive(FullCtor, Debug, Error)]
pub struct Error {
    message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
