use super::{Error, Result};
use crate::{ArbFile, ArbIndex};
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
            tracing::info!(key = %entry.key(), "translate");
            let text = entry.value().as_str().unwrap();
            let result = api
                .translate_text(text, options.target_lang.clone())
                .await?;
            let mut sentences = result.translations;
            if sentences.is_empty() {
                return Err(Error::NoTranslation(entry.key().to_string()));
            }
            let sentence = sentences.remove(0);
            output.insert_translation(entry.key(), sentence.text);
        } else {
            output.insert_entry(entry);
        }
    }
    Ok(output)
}
