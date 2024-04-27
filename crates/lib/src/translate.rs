use super::{Error, Result};
use crate::{ArbEntry, ArbFile, ArbIndex};
use deepl::{DeepLApi, Lang};
use std::path::{Path, PathBuf};

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
pub async fn translate(api: DeepLApi, options: TranslationOptions) -> Result<ArbFile> {
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

                let text = placeholders.names().join(",");
                let translated = translate_single_sentence(&api, &entry, &text, &options).await?;

                Some(
                    translated
                        .split(",")
                        .map(|s| s.to_owned())
                        .collect::<Vec<String>>(),
                )
            } else {
                None
            };

            let translated = translate_single_sentence(&api, &entry, text, &options).await?;

            let translation = if let Some(names) = names {
                let mut translation = String::new();
                for (index, name) in names.into_iter().enumerate() {
                    let needle = format!("{{{}}}", name);
                    let original = format!(
                        "{{{}}}",
                        placeholders.as_ref().unwrap().names().get(index).unwrap()
                    );
                    translation = translated.replace(&needle, &original);
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
    api: &DeepLApi,
    entry: &ArbEntry<'_>,
    text: &str,
    options: &TranslationOptions,
) -> Result<String> {
    let result = api
        .translate_text(text, options.target_lang.clone())
        .await?;
    let mut sentences = result.translations;
    if sentences.is_empty() {
        return Err(Error::NoTranslation(entry.key().to_string()));
    }
    let sentence = sentences.remove(0);
    Ok(sentence.text)
}
