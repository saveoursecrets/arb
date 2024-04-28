use std::path::PathBuf;
use thiserror::Error;

/// Error type for the library.
#[derive(Debug, Error)]
pub enum Error {
    /// Path has not parent.
    #[error("no parent for path '{0}'")]
    NoParentPath(PathBuf),

    /// Localizations index file does not contain the `arb-dir`.
    #[error("arb-dir is not defined in '{0}'")]
    ArbDirNotDefined(PathBuf),

    /// Localizations index file does not contain the `template-arb-file`.
    #[error("template-arb-file is not defined in '{0}'")]
    TemplateArbFileNotDefined(PathBuf),

    /// No YAML documents detected.
    #[error("no YAML documents in index file '{0}'")]
    NoYamlDocuments(PathBuf),

    /// API call did not return any translations for a key.
    #[error("no translation for '{0}'")]
    NoTranslation(String),

    /// Key is already prefixed.
    #[error("key '{0}' is already prefixed with an @ symbol")]
    AlreadyPrefixed(String),

    /// Placeholder defined in the JSON document does not exist
    /// in the string to be translated.
    #[error("placeholder '{0}' is declared but does not exist in source '{1}'")]
    PlaceholderNotDefined(String, String),

    /// IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// YAML error.
    #[error(transparent)]
    Yaml(#[from] yaml_rust2::ScanError),

    /// JSON error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// DeepL error.
    #[error(transparent)]
    Deepl(#[from] deepl::Error),
}
