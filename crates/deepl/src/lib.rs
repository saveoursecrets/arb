//! Client to call the DeepL API.
#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod api;
mod error;
mod lang;

pub use api::{
    ApiOptions, DeeplApi, Formality, Language, LanguageType, SplitSentences, TagHandling,
    TranslateTextRequest, TranslateTextResponse, Usage,
};
pub use error::Error;
pub use lang::Lang;

/// Result type for the library.
pub type Result<T> = std::result::Result<T, Error>;
