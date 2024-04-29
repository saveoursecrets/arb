use super::{Error, Result};
use crate::{ArbEntry, ArbFile, ArbIndex};
use deepl::{DeeplApi, Lang, TagHandling, TranslateTextRequest};
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

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
}

impl TranslationOptions {
    /// Create new translation options.
    pub fn new(path: impl AsRef<Path>, target_lang: Lang) -> Self {
        Self {
            index_file: path.as_ref().to_path_buf(),
            target_lang,
            dry_run: false,
            name_prefix: "app".to_string(),
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
}

#[derive(Debug)]
enum CachedEntry<'a> {
    /// Entry to passthrough to the output.
    Entry(ArbEntry<'a>),
    /// Entry to translate.
    Translate {
        entry: ArbEntry<'a>,
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

    for entry in entries {
        if entry.is_translatable() {
            let placeholders = template.placeholders(entry.key())?;
            tracing::info!(
              key = %entry.key(),
              placeholders = ?placeholders,
              "translate");

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

    if !translatable.is_empty() {
        let expected = translatable.len();

        let mut request = TranslateTextRequest::new(translatable, options.target_lang);
        request.tag_handling = Some(TagHandling::Xml);
        request.ignore_tags = Some(vec!["ph".to_string()]);

        let mut result = api.translate_text(&request).await?;

        if result.translations.len() != expected {
            return Err(Error::TranslationLength(
                expected,
                result.translations.len(),
            ));
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

    Ok(TranslateResult {
        index,
        template,
        translated: output,
    })
}
