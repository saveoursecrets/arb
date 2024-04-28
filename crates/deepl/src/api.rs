use crate::{Error, Lang, Result};
use reqwest::{Client, RequestBuilder};
use serde::{de::DeserializeOwned, ser::Serializer, Deserialize, Serialize};
use std::fmt;
use url::Url;

const ENDPOINT_FREE: &str = "https://api-free.deepl.com";
const ENDPOINT_PRO: &str = "https://api.deepl.com";

/// Enumeration of split sentence options.
#[derive(Debug, Serialize, Deserialize)]
pub enum SplitSentences {
    /// Do not split sentences.
    #[serde(rename = "0")]
    None,
    /// Split on punctuation and newlines.
    ///
    /// Default for XML tag handling.
    #[serde(rename = "1")]
    One,
    /// Split on punctuation only.
    ///
    /// Default for HTML tag handling.
    #[serde(rename = "nonewlines")]
    NoNewlines,
}

/// Variants for formality.
#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Formality {
    /// Default formality.
    #[default]
    Default,
    /// For a more formal language.
    More,
    /// For a more informal language.
    Less,
    /// For a more formal language if available,
    /// otherwise fallback to default formality.
    PreferMore,
    /// For a more informal language if available,
    /// otherwise fallback to default formality.
    PreferLess,
}

/// Supported language information.
#[derive(Debug, Serialize, Deserialize)]
pub struct Language {
    /// Language code.
    pub language: Lang,
    /// Language name.
    pub name: String,
    /// Whether the language supports formality.
    pub supports_formality: Option<bool>,
}

/// Enumeration of language types.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageType {
    /// Source language.
    #[default]
    Source,
    /// Target language.
    Target,
}

impl AsRef<str> for LanguageType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Source => "source",
            Self::Target => "target",
        }
    }
}

impl fmt::Display for LanguageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref(),)
    }
}

/// Account usage information.
#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    /// Character count.
    pub character_count: u64,
    /// Character limit.
    pub character_limit: u64,
}

/// Variants for tag handling.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TagHandling {
    // XML tag handling.
    Xml,
    // HTML tag handling.
    Html,
}

/// Single text translation.
#[derive(Debug, Serialize, Deserialize)]
pub struct TextTranslation {
    /// Translated text.
    pub text: String,
    /// Detected source language.
    pub detected_source_language: Lang,
}

/// Request to translate text.
#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateTextRequest {
    /// Text to translate.
    pub text: Vec<String>,
    /// Target language.
    pub target_lang: Lang,
    /// Tag handling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_handling: Option<TagHandling>,
    /// Source language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_lang: Option<Lang>,
    /// Context string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Preserve formatting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_formatting: Option<bool>,
    /// Glossary identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glossary_id: Option<String>,
    /// Outline detection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline_detection: Option<bool>,
    /// Non splitting tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_splitting_tags: Option<Vec<String>>,
    /// Splitting tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub splitting_tags: Option<Vec<String>>,
    /// Ignore tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_tags: Option<Vec<String>>,
    /// Formality.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formality: Option<Formality>,
    /// Split sentences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split_sentences: Option<SplitSentences>,
}

impl TranslateTextRequest {
    /// Create new translate text request.
    pub fn new(text: Vec<String>, target_lang: Lang) -> Self {
        Self {
            text,
            target_lang,
            source_lang: None,
            context: None,
            preserve_formatting: None,
            glossary_id: None,
            outline_detection: None,
            non_splitting_tags: None,
            splitting_tags: None,
            ignore_tags: None,
            tag_handling: None,
            formality: None,
            split_sentences: None,
        }
    }
}

/// Response to a translate text request.
#[derive(Debug, Serialize, Deserialize)]
pub struct TranslateTextResponse {
    /// Collection of translations.
    pub translations: Vec<TextTranslation>,
}

/// Options when creating an API endpoint.
pub struct ApiOptions {
    /// API key.
    api_key: String,
    /// Endpoint URL.
    endpoint: Url,
    /// Custom HTTP client.
    pub client: Option<Client>,
}

impl ApiOptions {
    /// API for the free endpoint.
    pub fn new_free(api_key: impl AsRef<str>) -> Self {
        Self {
            api_key: api_key.as_ref().to_owned(),
            endpoint: Url::parse(ENDPOINT_FREE).unwrap(),
            client: None,
        }
    }

    /// API for the pro endpoint.
    pub fn new_pro(api_key: impl AsRef<str>) -> Self {
        Self {
            api_key: api_key.as_ref().to_owned(),
            endpoint: Url::parse(ENDPOINT_PRO).unwrap(),
            client: None,
        }
    }
}

/// Interface to the DeepL API.
pub struct DeeplApi {
    client: Client,
    options: ApiOptions,
}

impl DeeplApi {
    /// Create a new DeepL API client.
    pub fn new(mut options: ApiOptions) -> Self {
        Self {
            client: options.client.take().unwrap_or_else(|| Client::new()),
            options,
        }
    }

    /// Get account usage.
    pub async fn usage(&self) -> Result<Usage> {
        let url = self.options.endpoint.join("v2/usage")?;
        let req = self.client.get(url);
        self.make_typed_request::<Usage>(req).await
    }

    /// Fetch supported languages.
    pub async fn languages(&self, lang_type: LanguageType) -> Result<Vec<Language>> {
        let mut url = self.options.endpoint.join("v2/languages")?;
        url.query_pairs_mut()
            .append_pair("type", lang_type.as_ref());
        let req = self.client.get(url);
        self.make_typed_request::<Vec<Language>>(req).await
    }

    /// Translate text.
    pub async fn translate_text(
        &self,
        request: &TranslateTextRequest,
    ) -> Result<TranslateTextResponse> {
        let url = self.options.endpoint.join("v2/translate")?;
        let req = self.client.post(url).json(request);
        self.make_typed_request::<TranslateTextResponse>(req).await
    }

    async fn make_typed_request<T: DeserializeOwned>(&self, req: RequestBuilder) -> Result<T> {
        let res = req
            .header(
                "Authorization",
                format!("DeepL-Auth-Key {}", self.options.api_key),
            )
            .send()
            .await?;
        res.error_for_status_ref()?;
        Ok(res.json::<T>().await?)
    }
}
