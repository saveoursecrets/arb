mod api;
mod error;
mod lang;

pub use api::{ApiOptions, DeeplApi, TagHandling, TranslateTextRequest, TranslateTextResponse};
pub use error::Error;
pub use lang::Lang;

pub type Result<T> = std::result::Result<T, Error>;
