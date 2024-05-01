//! Translate Flutter application resource bundles.
//!
//! # Examples
//!
//! Create a translation application resource bundle:
//!
//! ```
//! use arb_lib::{Intl, deepl::{DeeplApi, ApiOptions, Lang}};
//!
//! let api_key: std::env::var("DEEPL_API_KEY").unwrap();
//! let api = DeeplApi::new(ApiOptions::new(api_key));
//! let options = TranslationOptions::new(Lang::Fr);
//! let mut intl = Intl::new("l10n.yaml")?;
//! let result = intl.translate(&api, options).await?;
//! println!("{:#?}", result);
//! ```
#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod arb;
mod error;
mod intl;

pub use arb::*;
pub use error::Error;
pub use intl::*;

/// Result type for the library.
pub type Result<T> = std::result::Result<T, Error>;

pub use deepl;
