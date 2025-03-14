pub(crate) struct Error {
    message: String,
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
