use super::{Error, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
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
pub struct ArbFileContent(IndexMap<String, serde_json::Value>);
