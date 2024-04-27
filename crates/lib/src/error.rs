use std::path::PathBuf;
use thiserror::Error;

/// Error type for the library.
#[derive(Debug, Error)]
pub enum Error {
    #[error("no parent for path '{0}'")]
    NoParentPath(PathBuf),

    #[error("arb-dir is not defined in '{0}'")]
    ArbDirNotDefined(PathBuf),

    #[error("template-arb-file is not defined in '{0}'")]
    TemplateArbFileNotDefined(PathBuf),

    #[error("no YAML documents in index file '{0}'")]
    NoYamlDocuments(PathBuf),

    #[error("no translation for '{0}'")]
    NoTranslation(String),

    #[error("key '{0}' is already prefixed with an @ symbol")]
    AlreadyPrefixed(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Yaml(#[from] yaml_rust2::ScanError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Deepl(#[from] deepl::Error),
}
