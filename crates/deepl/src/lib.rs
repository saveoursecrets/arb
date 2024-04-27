mod api;
mod error;

pub use api::*;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

