//! Translate Flutter application resource bundles.
#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod error;
mod parser;
mod translate;

pub use error::Error;
pub use parser::*;
pub use translate::*;

/// Result type for the library.
pub type Result<T> = std::result::Result<T, Error>;

pub use deepl;
