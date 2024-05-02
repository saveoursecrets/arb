use super::{Error, Result};
use crate::{ArbEntry, ArbFile};
use deepl::{DeeplApi, Lang, TagHandling, TranslateTextRequest};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};
use yaml_rust2::YamlLoader;

const ARB_DIR: &str = "arb-dir";
const TEMPLATE_ARB_FILE: &str = "template-arb-file";
const NAME_PREFIX: &str = "name-prefix";
const OVERRIDES_DIR: &str = "overrides-dir";
const CACHE_FILE: &str = ".cache.json";

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

    /// Remove a cache entry.
    pub fn remove_entry(&mut self, lang: &Lang, key: &str) -> Option<Value> {
        if let Some(file) = self.0.get_mut(lang) {
            file.remove(key)
        } else {
            None
        }
    }
}

/// Variants for key invalidation.
pub enum Invalidation {
    /// Invalidate all keys.
    All,
    /// Invalidate specific keys.
    Keys(Vec<String>),
}

/// Options for translation.
pub struct TranslationOptions {
    /// Target language.
    pub target_lang: Lang,
    /// Whether this is a dry run.
    pub dry_run: bool,
    /// Invalidation configuration.
    pub invalidation: Option<Invalidation>,
    /// Overrides provided by humans.
    pub overrides: Option<HashMap<Lang, ArbFile>>,
    /// Disable updating the cache.
    ///
    /// Used in the test specs, you probably don't want
    /// to use this.
    #[doc(hidden)]
    pub disable_cache: bool,
}

impl TranslationOptions {
    /// Create new translation options.
    pub fn new(target_lang: Lang) -> Self {
        Self {
            target_lang,
            dry_run: false,
            invalidation: None,
            overrides: None,
            disable_cache: false,
        }
    }
}

/// Translate result.
#[derive(Debug)]
pub struct TranslateResult {
    /// Template information.
    pub template: ArbFile,
    /// Translated content.
    pub translated: ArbFile,
    /// Number of translations.
    pub length: usize,
}

#[derive(Debug)]
enum CachedEntry<'a> {
    /// Entry to passthrough to the output.
    ///
    /// Typically used for meta data or comments prefixed
    /// with the @ symbol.
    Entry(ArbEntry<'a>),
    /// Entry to translate.
    Translate {
        entry: ArbEntry<'a>,
        /// Names of the placeholders.
        names: Option<Vec<&'a str>>,
        /// Specific index to insert.
        index: Option<usize>,
    },
}

/// Internationalization index file.
///
/// Translations are loaded by convention from the directory pointed
/// to by the `arb-dir`. The default convention is to use `app` as the
/// prefix concatenated with a lowercase language identifier delimited
/// by an underscore. Language identifiers in file names should use
/// underscores and ***not hyphens***. For example, the file name for
/// the `EN-US` language would be `app_en_us.arb`.
#[derive(Debug)]
pub struct Intl {
    file_path: PathBuf,
    arb_dir: String,
    template_language: Lang,
    template_arb_file: String,
    name_prefix: String,
    overrides_dir: Option<String>,
    pub(crate) cache: ArbCache,
}

impl Intl {
    /// Load the YAML file using the default file name prefix.
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        Self::new_with_prefix(path, None)
    }

    /// Load the YAML file with a given file name prefix.
    pub fn new_with_prefix(path: impl AsRef<Path>, name_prefix: Option<String>) -> Result<Self> {
        if !path.as_ref().try_exists()? {
            return Err(Error::NoFile(path.as_ref().to_path_buf()));
        }

        if !path.as_ref().is_file() {
            return Err(Error::NotFile(path.as_ref().to_path_buf()));
        }

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

        let name_prefix = if let Some(name_prefix) = doc[NAME_PREFIX].as_str() {
            name_prefix.to_string()
        } else {
            name_prefix.unwrap_or_else(|| "app".to_string())
        };

        let overrides_dir = doc[OVERRIDES_DIR].as_str().map(|s| s.to_string());

        let stem = template_arb_file.trim_end_matches(".arb");
        let pat = format!("{}_", name_prefix);
        let lang_code = stem.trim_start_matches(&pat);
        let template_language: Lang = lang_code.parse()?;

        let mut index = Intl {
            file_path: path.as_ref().to_owned(),
            arb_dir: arb_dir.to_owned(),
            template_arb_file: template_arb_file.to_owned(),
            template_language,
            name_prefix,
            cache: Default::default(),
            overrides_dir,
        };
        index.cache = index.read_cache()?;

        Ok(index)
    }

    /// Directory for application resource bundles.
    pub fn arb_dir(&self) -> &str {
        &self.arb_dir
    }

    /// Template application resource bundle.
    pub fn template_arb_file(&self) -> &str {
        &self.template_arb_file
    }

    /// Prefix used to compute file names.
    pub fn name_prefix(&self) -> &str {
        &self.name_prefix
    }

    /// Directory for override files.
    pub fn overrides_dir(&self) -> Option<&str> {
        self.overrides_dir.as_ref().map(|s| &s[..])
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

        if !parent.is_dir() {
            return Err(Error::NotDirectory(parent));
        }

        Ok(parent)
    }

    /// List translations in the configured `arb-dir`.
    pub fn list_translated(&self) -> Result<BTreeMap<Lang, PathBuf>> {
        self.list_directory(self.arb_directory()?)
    }

    /// List translated languages in a directory.
    pub fn list_directory(&self, dir: impl AsRef<Path>) -> Result<BTreeMap<Lang, PathBuf>> {
        let mut output = BTreeMap::new();

        if !dir.as_ref().is_dir() {
            return Err(Error::NotDirectory(dir.as_ref().to_path_buf()));
        }

        for entry in std::fs::read_dir(dir.as_ref())? {
            let entry = entry?;
            let path = entry.path();
            if let (true, Some(lang)) = (path.is_file(), self.parse_file_name(&path)) {
                output.insert(lang, path);
            }
        }
        Ok(output)
    }

    /// Attempt to load override definitions.
    ///
    /// If a languages list is given only load the
    /// given languages.
    pub fn load_overrides(
        &self,
        dir: impl AsRef<Path>,
        languages: Option<Vec<Lang>>,
    ) -> Result<HashMap<Lang, ArbFile>> {
        let mut output = HashMap::new();
        let langs = self.list_directory(dir.as_ref())?;
        for (lang, path) in langs {
            if let Some(filters) = &languages {
                if !filters.contains(&lang) {
                    continue;
                }
            }
            let content = std::fs::read_to_string(&path)?;
            let file: ArbFile = serde_json::from_str(&content)?;
            output.insert(lang, file);
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

    /// Translate to a target language.
    ///
    /// Placeholders are converted to XML tags and ignored from
    /// translation to preserve the placeholder names.
    pub async fn translate(
        &mut self,
        api: &DeeplApi,
        options: TranslationOptions,
    ) -> Result<TranslateResult> {
        tracing::info!(lang = %options.target_lang, "translate");

        let template = self.template_content()?;
        let mut output = self.load_or_default(options.target_lang)?;
        let mut cached = Vec::new();
        let mut translatable = Vec::new();
        let diff = template.diff(&output, self.cache.get_file(&options.target_lang));

        let overrides = if let Some(overrides) = &options.overrides {
            overrides.get(&options.target_lang)
        } else {
            None
        };

        for entry in template.entries() {
            let invalidated = match &options.invalidation {
                Some(Invalidation::All) => true,
                Some(Invalidation::Keys(keys)) => keys.iter().any(|x| x == entry.key().as_ref()),
                _ => false,
            };

            // Ignore if removed or not in the set of added keys.
            if !invalidated
                && (diff.delete.contains(entry.key().as_ref())
                    || (!diff.create.contains(entry.key().as_ref())
                        && !diff.update.contains(entry.key().as_ref())))
            {
                continue;
            }

            // Would be overwritten by a manual translation
            // so no need to translate
            if let Some(overrides) = overrides {
                if overrides.lookup(entry.key().as_ref()).is_some() {
                    continue;
                }
            }

            if entry.is_translatable() {
                let placeholders = template.placeholders(entry.key())?;
                if let Some(placeholders) = &placeholders {
                    tracing::info!(
                      key = %entry.key(),
                      placeholders = ?placeholders.to_vec(),
                      "prepare");
                } else {
                    tracing::info!(
                      key = %entry.key(),
                      "prepare");
                }

                let text = entry.value().as_str().unwrap();

                // Verify the source placeholders are declared correctly
                let names = if let Some(placeholders) = &placeholders {
                    placeholders.verify(text)?;
                    Some(placeholders.to_vec())
                } else {
                    None
                };

                // Replace placeholders with XML tags
                let text = if let Some(names) = &names {
                    let mut text = text.to_string();
                    for name in names {
                        text = text.replacen(
                            &format!("{{{}}}", name),
                            &format!("<ph>{}</ph>", name),
                            1,
                        );
                    }
                    Cow::Owned(text)
                } else {
                    Cow::Borrowed(text)
                };

                let key_index = if diff.create.contains(entry.key().as_ref()) {
                    template.contents.get_index_of(entry.key().as_ref())
                } else {
                    None
                };

                if !options.dry_run {
                    translatable.push(text.as_ref().to_string());
                    if !options.disable_cache {
                        self.cache.add_entry(options.target_lang, entry.clone());
                    }
                    cached.push(CachedEntry::Translate {
                        entry,
                        names,
                        index: key_index,
                    });
                } else {
                    cached.push(CachedEntry::Entry(entry));
                }
            } else {
                cached.push(CachedEntry::Entry(entry));
            }
        }

        // Clean up any existing entries scheduled to be deleted
        for key in diff.delete {
            tracing::info!(key = %key, "delete");
            output.remove(&key);
            self.cache.remove_entry(&options.target_lang, &key);
        }

        let length = translatable.len();

        tracing::info!(
            lang = %options.target_lang,
            length = %length,
            "translate");

        if !translatable.is_empty() {
            let mut request = TranslateTextRequest::new(translatable, options.target_lang);
            request.tag_handling = Some(TagHandling::Xml);
            request.ignore_tags = Some(vec!["ph".to_string()]);

            let mut result = api.translate_text(&request).await?;

            if result.translations.len() != length {
                return Err(Error::TranslationLength(length, result.translations.len()));
            }

            for entry in cached {
                match entry {
                    CachedEntry::Entry(entry) => {
                        output.insert_entry(entry);
                    }
                    CachedEntry::Translate {
                        entry,
                        names,
                        index,
                    } => {
                        let translated = result.translations.remove(0).text;

                        // Revert placeholder XML tags
                        let translation = if let Some(names) = names {
                            let mut translation = translated;
                            for name in names.into_iter() {
                                let needle = format!("<ph>{}</ph>", name);
                                let original = format!("{{{}}}", name);
                                translation = translation.replacen(&needle, &original, 1);
                            }
                            translation
                        } else {
                            translated
                        };

                        if let Some(index) = index {
                            if index < output.len() {
                                output.shift_insert_translation(index, entry.key(), translation)
                            } else {
                                output.insert_translation(entry.key(), translation)
                            }
                        } else {
                            output.insert_translation(entry.key(), translation)
                        }
                    }
                }
            }
        }

        if let Some(overrides) = overrides {
            for entry in overrides.entries() {
                tracing::info!(key = %entry.key().as_ref(), "override");
                output.insert_entry(entry);
            }
        }

        // Update the cache file
        if !options.disable_cache {
            self.write_cache()?;
        }

        Ok(TranslateResult {
            template,
            translated: output,
            length,
        })
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

    fn write_cache(&self) -> Result<()> {
        let cache_path = self.arb_directory()?.join(CACHE_FILE);
        let mut cache_file = std::fs::File::create(cache_path)?;
        serde_json::to_writer_pretty(&mut cache_file, &self.cache)?;
        Ok(())
    }
}
