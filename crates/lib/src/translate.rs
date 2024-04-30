use super::{Error, Result};
use crate::{ArbEntry, ArbFile, ArbIndex};
use deepl::{DeeplApi, Lang, TagHandling, TranslateTextRequest};
use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Variants for key invalidation.
pub enum Invalidation {
    /// Invalidate all keys.
    All,
    /// Invalidate specific keys.
    Keys(Vec<String>),
}

/// Options for translation.
pub struct TranslationOptions {
    /// YAML localization index file.
    pub index_file: PathBuf,
    /// Target language.
    pub target_lang: Lang,
    /// Whether this is a dry run.
    pub dry_run: bool,
    /// Prefix for localization file names.
    pub name_prefix: String,
    /// Invalidation configuration.
    pub invalidation: Option<Invalidation>,
    /// Overrides provided by humans.
    pub overrides: Option<HashMap<Lang, ArbFile>>,
}

impl TranslationOptions {
    /// Create new translation options.
    pub fn new(path: impl AsRef<Path>, target_lang: Lang) -> Self {
        Self {
            index_file: path.as_ref().to_path_buf(),
            target_lang,
            dry_run: false,
            name_prefix: "app".to_string(),
            invalidation: None,
            overrides: None,
        }
    }
}

/// Translate result.
pub struct TranslateResult {
    /// Localizations index.
    pub index: ArbIndex,
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
    },
}

/// Translate to a target language.
///
/// Placeholders are converted to XML tags and ignored from
/// translation to preserve the placeholder names.
pub async fn translate(api: DeeplApi, options: TranslationOptions) -> Result<TranslateResult> {
    let index = ArbIndex::parse_yaml(&options.index_file, &options.name_prefix)?;
    let template = index.template_content()?;
    let entries = template.entries();
    let mut output = index.load_or_default(options.target_lang)?;
    let mut cached = Vec::new();
    let mut translatable = Vec::new();
    let diff = template.diff(&output);

    let overrides = if let Some(overrides) = &options.overrides {
        overrides.get(&options.target_lang)
    } else {
        None
    };

    for entry in entries {
        let invalidated = match &options.invalidation {
            Some(Invalidation::All) => true,
            Some(Invalidation::Keys(keys)) => keys.iter().any(|x| x == entry.key().as_ref()),
            _ => false,
        };

        // Ignore if removed or not in the set of added keys.
        if !invalidated
            && (diff.delete.contains(entry.key().as_ref())
                || !diff.create.contains(entry.key().as_ref()))
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
            tracing::info!(
              key = %entry.key(),
              placeholders = ?placeholders,
              "prepare");

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
                    text =
                        text.replacen(&format!("{{{}}}", name), &format!("<ph>{}</ph>", name), 1);
                }
                Cow::Owned(text)
            } else {
                Cow::Borrowed(text)
            };

            if !options.dry_run {
                translatable.push(text.as_ref().to_string());
                cached.push(CachedEntry::Translate { entry, names });
            } else {
                cached.push(CachedEntry::Entry(entry));
            }
        } else {
            cached.push(CachedEntry::Entry(entry));
        }
    }

    for key in diff.delete {
        tracing::info!(key = %key, "delete");
        output.remove(&key);
    }

    let length = translatable.len();
    if !translatable.is_empty() {
        tracing::info!(
          lang = %options.target_lang,
          length = %length,
          "translate");

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
                CachedEntry::Translate { entry, names } => {
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

                    output.insert_translation(entry.key(), translation)
                }
            }
        }
    }

    if let Some(overrides) = overrides {
        for entry in overrides.entries() {
            output.insert_entry(entry);
        }
    }

    Ok(TranslateResult {
        index,
        template,
        translated: output,
        length,
    })
}
