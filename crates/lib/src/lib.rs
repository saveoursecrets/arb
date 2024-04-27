mod error;
mod parser;

pub use error::Error;
pub use parser::*;

/// Result type for the library.
pub type Result<T> = std::result::Result<T, Error>;
