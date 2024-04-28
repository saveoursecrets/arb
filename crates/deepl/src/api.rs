use crate::{Error, Lang, Result};
use reqwest::{Client, RequestBuilder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use url::Url;

const ENDPOINT_FREE: &str = "https://api-free.deepl.com";
const ENDPOINT_PRO: &str = "https://api.deepl.com";

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