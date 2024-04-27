use super::{Error, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

const ARB_DIR: &str = "arb-dir";
const TEMPLATE_ARB_FILE: &str = "template-arb-file";

/// Internationalization index file.
#[derive(Debug)]
pub struct ArbIndex {
    file_path: PathBuf,
    arb_dir: String,
    template_arb_file: String,
}

impl ArbIndex {
    /// Directory for application resource bundles.
    pub fn arb_dir(&self) -> &str {
        &self.arb_dir
    }

    /// Template application resource bundle.
    pub fn template_arb_file(&self) -> &str {
        &self.template_arb_file
    }

    /// Load and parse the template application resource bundle.
    pub fn template_content(&self) -> Result<ArbFileContent> {
        let path = self
            .parent_path()?
            .to_owned()
            .join(&self.arb_dir)
            .join(&self.template_arb_file);

        let content = std::fs::read_to_string(&path)?;
        let arb: ArbFileContent = serde_json::from_str(&content)?;
        Ok(arb)
    }

    /// Compute the parent of the index file.
    fn parent_path(&self) -> Result<&Path> {
        self.file_path
            .parent()
            .ok_or_else(|| Error::NoParentPath(self.file_path.clone()))
    }

    /// Parse a YAML index file.
    pub fn parse_yaml(path: impl AsRef<Path>) -> Result<Self> {
        use yaml_rust2::YamlLoader;
        let content = std::fs::read_to_string(path.as_ref())?;
        let docs = YamlLoader::load_from_str(&content)?;

        if docs.is_empty() {
            return Err(Error::NoYamlDocuments);
        }

        let doc = &docs[0];
        let arb_dir = doc[ARB_DIR]
            .as_str()
            .ok_or_else(|| Error::ArbDirNotDefined(path.as_ref().to_owned()))?;

        let template_arb_file = doc[TEMPLATE_ARB_FILE]
            .as_str()
            .ok_or_else(|| Error::TemplateArbFileNotDefined(path.as_ref().to_owned()))?;

        Ok(ArbIndex {
            file_path: path.as_ref().to_owned(),
            arb_dir: arb_dir.to_owned(),
            template_arb_file: template_arb_file.to_owned(),
        })
    }
}

/// Content of an application resource bundle file.
#[derive(Debug, Serialize, Deserialize)]
pub struct ArbFileContent(IndexMap<String, Value>);

impl ArbFileContent {
    /// All of the application resource bundle entries.
    pub fn entries(&self) -> Vec<ArbEntry<'_>> {
        self.0
            .iter()
            .map(|(k, v)| ArbEntry(ArbKey(k), ArbValue(v)))
            .collect()
    }

    /// Lookup an entry by key.
    pub fn lookup<'a>(&'a self, key: &'a str) -> Option<ArbEntry<'a>> {
        self.0.get(key).map(|v| ArbEntry(ArbKey(key), ArbValue(v)))
    }
}

/// Entry in an application resource bundle map.
pub struct ArbEntry<'a>(ArbKey<'a>, ArbValue<'a>);

impl<'a> ArbEntry<'a> {
    /// Key for the entry.
    pub fn key(&self) -> &ArbKey<'a> {
        &self.0
    }

    /// Value of the entry.
    pub fn value(&self) -> &ArbValue<'a> {
        &self.1
    }

    /// Determine if this entry is translatable.
    ///
    /// An entry is only translatable when the key is not prefixed
    /// with an @ symbol and the value is of the string type.
    pub fn is_translatable(&self) -> bool {
        self.0.is_translatable() && self.1.is_translatable()
    }
}

/// Key in the application resource bundle map.
pub struct ArbKey<'a>(&'a str);

impl<'a> ArbKey<'a> {
    /// Determine if this key is prefixed with the @ symbol.
    ///
    /// The @ symbol is used to declare meta data for translatable
    /// keys (such as placeholders) or for comments.
    pub fn is_prefixed(&self) -> bool {
        self.0.starts_with('@')
    }

    /// Determine if this key is translatable.
    fn is_translatable(&self) -> bool {
        !self.is_prefixed()
    }
}

/// Value in the application resource bundle map.
pub struct ArbValue<'a>(&'a Value);

impl<'a> ArbValue<'a> {
    /// String reference, only available when translatable.
    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(val) = self.0 {
            Some(val)
        } else {
            None
        }
    }

    /// Determine if this value is translatable.
    fn is_translatable(&self) -> bool {
        matches!(self.0, Value::String(_))
    }
}
