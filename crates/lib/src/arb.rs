use super::{Error, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashSet, fmt};

const PLACEHOLDERS: &str = "placeholders";

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
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
