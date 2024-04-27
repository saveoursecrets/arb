mod error;
pub mod parser;

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

