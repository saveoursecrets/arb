use super::{Error, Result};
use deepl::Lang;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashSet,
    fmt,
    path::{Path, PathBuf},
};

const ARB_DIR: &str = "arb-dir";
const TEMPLATE_ARB_FILE: &str = "template-arb-file";
const PLACEHOLDERS: &str = "placeholders";

/// Internationalization index file.
#[derive(Debug)]
pub struct ArbIndex {
    file_path: PathBuf,
    arb_dir: String,
    template_arb_file: String,
    name_prefix: String,
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
    pub fn template_content(&self) -> Result<ArbFile> {
        let path = self
            .parent_path()?
            .to_owned()
            .join(&self.arb_dir)
            .join(&self.template_arb_file);

        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Compute the parent of the index file.
    pub fn parent_path(&self) -> Result<&Path> {
        self.file_path
            .parent()
            .ok_or_else(|| Error::NoParentPath(self.file_path.clone()))
    }

    /// Parse a YAML index file.
    pub fn parse_yaml(path: impl AsRef<Path>, name_prefix: impl AsRef<str>) -> Result<Self> {
        use yaml_rust2::YamlLoader;
        let content = std::fs::read_to_string(path.as_ref())?;
        let docs = YamlLoader::load_from_str(&content)?;

        if docs.is_empty() {
            return Err(Error::NoYamlDocuments(path.as_ref().to_owned()));
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
            name_prefix: name_prefix.as_ref().to_string(),
        })
    }

    /// Path to a language file.
    pub fn file_path(&self, lang: Lang) -> Result<PathBuf> {
        let output_file = format!(
            "{}_{}.arb",
            self.name_prefix,
            lang.to_string().to_lowercase().replace("-", "_")
        );
        Ok(self.parent_path()?.join(output_file))
    }

    /// Load a language file from disc.
    pub fn load(&self, lang: Lang) -> Result<ArbFile> {
        let path = self.file_path(lang)?;
        if !path.try_exists()? {
            return Err(Error::NoFile(path));
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// Load a language file if it exists otherwise use an
    /// empty file.
    pub fn load_or_default(&self, lang: Lang) -> Result<ArbFile> {
        Ok(self.load(lang).ok().unwrap_or_default())
    }
}

/// Diff of the keys in two language files.
#[derive(Debug, Serialize, Deserialize)]
pub struct FileDiff {
    /// Set of keys that exist in the template but
    /// not in the target language.
    pub create: HashSet<String>,
    /// Set of keys that exist in the target language
    /// but not in the template.
    pub delete: HashSet<String>,
}

/// Content of an application resource bundle file.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArbFile(IndexMap<String, Value>);

impl ArbFile {
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

    /// Insert a translated value.
    pub fn insert_translation<'a>(&mut self, key: &ArbKey<'a>, text: String) {
        self.0.insert(key.to_string(), Value::String(text));
    }

    /// Insert an entry.
    pub fn insert_entry<'a>(&mut self, entry: ArbEntry<'a>) {
        self.0.insert(entry.key().to_string(), entry.value().into());
    }

    /// Remove an entry.
    pub fn remove(&mut self, key: &str) {
        self.0.remove(key);
    }

    /// Attempt to locate the placeholder names for a key.
    pub fn placeholders<'a>(&self, key: &ArbKey<'a>) -> Result<Option<Placeholders<'_>>> {
        if key.as_ref().starts_with('@') {
            return Err(Error::AlreadyPrefixed(key.to_string()));
        }

        let meta_key = format!("@{}", key.as_ref());
        if let Some(value) = self.0.get(&meta_key) {
            if let Value::Object(map) = value {
                if let Some(Value::Object(placeholders)) = map.get(PLACEHOLDERS) {
                    let keys = placeholders.keys().map(|k| &k[..]).collect::<Vec<_>>();
                    Ok(Some(Placeholders::new(keys)))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Get a diff of keys between files.
    pub fn diff<'a>(&'a self, other: &'a ArbFile) -> FileDiff {
        let lhs = self.0.keys().collect::<HashSet<_>>();
        let rhs = other.0.keys().collect::<HashSet<_>>();
        let create = lhs
            .difference(&rhs)
            .map(|s| s.to_string())
            .collect::<HashSet<_>>();
        let delete = rhs
            .difference(&lhs)
            .map(|s| s.to_string())
            .collect::<HashSet<_>>();
        FileDiff { create, delete }
    }
}

/// Entry in an application resource bundle map.
#[derive(Debug)]
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
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ArbKey<'a>(&'a str);

impl<'a> ArbKey<'a> {
    /// Create a new key.
    pub fn new(key: &'a str) -> Self {
        Self(key)
    }

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

impl<'a> AsRef<str> for ArbKey<'a> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a> fmt::Display for ArbKey<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Value in the application resource bundle map.
#[derive(Debug, Eq, PartialEq)]
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

impl<'a> From<&ArbValue<'a>> for Value {
    fn from(value: &ArbValue<'a>) -> Self {
        value.0.clone()
    }
}

impl<'a> From<&'a Value> for ArbValue<'a> {
    fn from(value: &'a Value) -> Self {
        ArbValue(value)
    }
}

/// Collection of placeholder names.
#[derive(Debug)]
pub struct Placeholders<'a>(Vec<&'a str>);

impl<'a> Placeholders<'a> {
    /// Create new placeholders.
    pub fn new(names: Vec<&'a str>) -> Self {
        Self(names)
    }

    /// Slice of placeholder names.
    pub fn names(&self) -> &[&'a str] {
        self.0.as_slice()
    }

    /// Convert to a vector of string slices.
    pub fn to_vec(&self) -> Vec<&'a str> {
        self.0.clone()
    }

    /// Verify that a source string contains all the referenced
    /// placeholders.
    pub fn verify(&self, source: &str) -> Result<()> {
        for name in &self.0 {
            let needle = format!("{{{}}}", name);
            if !source.contains(&*needle) {
                return Err(Error::PlaceholderNotDefined(
                    name.to_string(),
                    source.to_string(),
                ));
            }
        }
        Ok(())
    }
}
