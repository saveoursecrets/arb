mod error;
mod parser;
mod translate;

pub use error::Error;
pub use parser::*;
pub use translate::*;

pub type Result<T> = std::result::Result<T, Error>;

pub use deepl;
