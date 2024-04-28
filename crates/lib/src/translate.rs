use super::{Error, Result};
use crate::{ArbEntry, ArbFile, ArbIndex};
use deepl::{DeeplApi, Lang, TagHandling, TranslateTextRequest};
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

/// Options for translation.
pub struct TranslationOptions {
    index_file: PathBuf,
    target_lang: Lang,
}

impl TranslationOptions {
    /// Create new translation options.
    pub fn new(path: impl AsRef<Path>, target_lang: Lang) -> Self {
        Self {
            index_file: path.as_ref().to_path_buf(),
            target_lang,
        }
    }
}

/// Translate to a target language.
pub async fn translate(api: DeeplApi, options: TranslationOptions) -> Result<ArbFile> {
    let index = ArbIndex::parse_yaml(&options.index_file)?;
    let template = index.template_content()?;
    let entries = template.entries();
    let mut output = ArbFile::default();
    for entry in entries {
        if entry.is_translatable() {
            let placeholders = template.placeholders(entry.key())?;
            tracing::info!(
              key = %entry.key(), placeholders = ?placeholders, "translate");

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

            let translated =
                translate_single_sentence(&api, &entry, text.as_ref(), &options).await?;

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

            output.insert_translation(entry.key(), translation);
        } else {
            output.insert_entry(entry);
        }
    }
    Ok(output)
}

async fn translate_single_sentence(
    api: &DeeplApi,
    entry: &ArbEntry<'_>,
    text: &str,
    options: &TranslationOptions,
) -> Result<String> {
    let mut request = TranslateTextRequest::new(vec![text.to_string()], options.target_lang);
    request.tag_handling = Some(TagHandling::Xml);
    request.ignore_tags = Some(vec!["ph".to_string()]);
    let result = api.translate_text(&request).await?;
    let mut sentences = result.translations;
    if sentences.is_empty() {
        return Err(Error::NoTranslation(entry.key().to_string()));
    }
    let sentence = sentences.remove(0);
    Ok(sentence.text)
}
