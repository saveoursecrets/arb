use super::{Error, Result};
use deepl::Lang;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashSet},
    fmt,
    path::{Path, PathBuf},
};

const ARB_DIR: &str = "arb-dir";
const TEMPLATE_ARB_FILE: &str = "template-arb-file";
const PLACEHOLDERS: &str = "placeholders";
const CACHE_FILE: &str = ".cache.arb";

/// Cache of template strings used for translations.
///
/// Used to determine which keys need updating when strings
/// in the template file are changed.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArbCache(BTreeMap<Lang, ArbFile>);

impl ArbCache {
    /// Get an application resource bundle file.
    pub fn get_file(&self, lang: &Lang) -> Option<&ArbFile> {
        self.0.get(lang)
    }

    /// Add a cache entry.
    pub fn add_entry(&mut self, lang: Lang, entry: ArbEntry<'_>) {
        let file = self.0.entry(lang).or_insert(ArbFile::default());
        file.insert_entry(entry);
    }

    /// Get a cache entry.
    pub fn get_entry<'a>(&'a self, lang: &Lang, key: &'a str) -> Option<ArbEntry<'a>> {
        if let Some(file) = self.0.get(lang) {
            file.lookup(key)
        } else {
            None
        }
    }

    /// Remove a cache entry.
    pub fn remove_entry(&mut self, lang: &Lang, key: &str) -> Option<Value> {
        if let Some(file) = self.0.get_mut(lang) {
            file.remove(key)
        } else {
            None
        }
    }
}

/// Internationalization index file.
#[derive(Debug)]
pub struct ArbIndex {
    file_path: PathBuf,
    arb_dir: String,
    template_language: Lang,
    template_arb_file: String,
    name_prefix: String,
    pub(crate) cache: ArbCache,
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

    /// Language of the template application resource bundle.
    pub fn template_language(&self) -> &Lang {
        &self.template_language
    }

    /// Get the cache of original translations.
    pub fn cache(&self) -> &ArbCache {
        &self.cache
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

        let stem = template_arb_file.trim_end_matches(".arb");
        let pat = format!("{}_", name_prefix.as_ref());
        let lang_code = stem.trim_start_matches(&pat);
        let template_language: Lang = lang_code.parse()?;

        let mut index = ArbIndex {
            file_path: path.as_ref().to_owned(),
            arb_dir: arb_dir.to_owned(),
            template_arb_file: template_arb_file.to_owned(),
            template_language,
            name_prefix: name_prefix.as_ref().to_string(),
            cache: Default::default(),
        };
        index.cache = index.read_cache()?;

        Ok(index)
    }

    /// Path to a language file.
    pub fn file_path(&self, lang: Lang) -> Result<PathBuf> {
        Ok(self.arb_directory()?.join(self.format_file_name(lang)))
    }

    /// Format a language to a file name.
    pub fn format_file_name(&self, lang: Lang) -> String {
        format!(
            "{}_{}.arb",
            self.name_prefix,
            lang.to_string().to_lowercase().replace("-", "_")
        )
    }

    /// Parse a file path to a language.
    pub fn parse_file_name(&self, path: impl AsRef<Path>) -> Option<Lang> {
        if let Some(name) = path.as_ref().file_stem() {
            let name = name.to_string_lossy();
            if name.starts_with(&self.name_prefix) {
                let pat = format!("{}_", self.name_prefix);
                let lang_code = name.trim_start_matches(&pat);
                lang_code.parse().ok()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Compute the application resource bundle directory relative to the
    /// parent of the internationalization index file.
    pub fn arb_directory(&self) -> Result<PathBuf> {
        let arb_dir = PathBuf::from(&self.arb_dir);
        let parent = if arb_dir.is_relative() {
            self.parent_path()?.join(arb_dir)
        } else {
            arb_dir
        };
        Ok(parent)
    }

    /// List translated languages.
    pub fn list_translated(&self) -> Result<BTreeMap<Lang, PathBuf>> {
        let mut output = BTreeMap::new();
        let dir = self.arb_directory()?;
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if let (true, Some(lang)) = (path.is_file(), self.parse_file_name(&path)) {
                output.insert(lang, path);
            }
        }
        Ok(output)
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
        match self.load(lang) {
            Ok(res) => Ok(res),
            Err(Error::NoFile(_)) => Ok(ArbFile::default()),
            Err(e) => Err(e),
        }
    }

    fn read_cache(&self) -> Result<ArbCache> {
        let cache_path = self.arb_directory()?.join(CACHE_FILE);
        if cache_path.try_exists()? {
            let mut cache_file = std::fs::File::open(cache_path)?;
            Ok(serde_json::from_reader(&mut cache_file)?)
        } else {
            Ok(ArbCache::default())
        }
    }

    pub(super) fn write_cache(&self) -> Result<()> {
        let cache_path = self.arb_directory()?.join(CACHE_FILE);
        let mut cache_file = std::fs::File::create(cache_path)?;
        serde_json::to_writer_pretty(&mut cache_file, &self.cache)?;
        Ok(())
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
    /// Set of keys that have changed in the template
    /// since the last translation.
    pub update: HashSet<String>,
}

/// Content of an application resource bundle file.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ArbFile {
    #[serde(flatten)]
    pub(crate) contents: IndexMap<String, Value>,
}

impl ArbFile {
    /// Number of entries.
    pub fn len(&self) -> usize {
        self.contents.len()
    }

    /// Whether this application resource bundle is empty.
    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }

    /// All of the application resource bundle entries.
    pub fn entries(&self) -> Vec<ArbEntry<'_>> {
        self.contents
            .iter()
            .map(|(k, v)| ArbEntry(ArbKey(k), ArbValue(v)))
            .collect()
    }

    /// Lookup an entry by key.
    pub fn lookup<'a>(&'a self, key: &'a str) -> Option<ArbEntry<'a>> {
        self.contents
            .get(key)
            .map(|v| ArbEntry(ArbKey(key), ArbValue(v)))
    }

    /// Insert a translated value.
    pub fn insert_translation<'a>(&mut self, key: &ArbKey<'a>, text: String) {
        self.contents.insert(key.to_string(), Value::String(text));
    }

    /// Shift insert a translated value.
    pub fn shift_insert_translation<'a>(&mut self, index: usize, key: &ArbKey<'a>, text: String) {
        self.contents
            .shift_insert(index, key.to_string(), Value::String(text));
    }

    /// Insert an entry.
    pub fn insert_entry<'a>(&mut self, entry: ArbEntry<'a>) {
        self.contents
            .insert(entry.key().to_string(), entry.value().into());
    }

    /// Remove an entry.
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.contents.shift_remove(key)
    }

    /// Attempt to locate the placeholder names for a key.
    pub fn placeholders<'a>(&self, key: &ArbKey<'a>) -> Result<Option<Placeholders<'_>>> {
        if key.as_ref().starts_with('@') {
            return Err(Error::AlreadyPrefixed(key.to_string()));
        }

        let meta_key = format!("@{}", key.as_ref());
        if let Some(value) = self.contents.get(&meta_key) {
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
    pub fn diff<'a>(&'a self, other: &'a ArbFile, cache: Option<&'a ArbFile>) -> FileDiff {
        let lhs = self.contents.keys().collect::<HashSet<_>>();
        let rhs = other.contents.keys().collect::<HashSet<_>>();
        let create = lhs
            .difference(&rhs)
            .map(|s| s.to_string())
            .collect::<HashSet<_>>();
        let delete = rhs
            .difference(&lhs)
            .map(|s| s.to_string())
            .collect::<HashSet<_>>();
        let mut update = HashSet::new();
        if let Some(cache) = cache {
            for entry in cache.entries() {
                if let (Some(current), Some(cached)) = (
                    self.contents.get(entry.key().as_ref()),
                    cache.contents.get(entry.key().as_ref()),
                ) {
                    if current != cached {
                        update.insert(entry.key().as_ref().to_string());
                    }
                }
            }
        }
        FileDiff {
            create,
            delete,
            update,
        }
    }
}

/// Entry in an application resource bundle map.
#[derive(Debug, Clone)]
pub struct ArbEntry<'a>(ArbKey<'a>, ArbValue<'a>);

impl<'a> ArbEntry<'a> {
    /// Create a new entry.
    pub fn new(key: &'a str, value: &'a Value) -> Self {
        Self(ArbKey::new(key), ArbValue::new(value))
    }

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
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ArbValue<'a>(&'a Value);

impl<'a> ArbValue<'a> {
    /// Create a new value.
    pub fn new(value: &'a Value) -> Self {
        Self(value)
    }

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
