use anyhow::Result;
use arb_lib::deepl::{ApiOptions, DeeplApi, LanguageType};

#[tokio::test]
pub async fn languages_source() -> Result<()> {
    let api = DeeplApi::new(ApiOptions::new_free(
        &std::env::var("DEEPL_API_KEY").unwrap(),
    ));
    let langs = api.languages(Default::default()).await?;
    assert!(!langs.is_empty());
    Ok(())
}

#[tokio::test]
pub async fn languages_target() -> Result<()> {
    let api = DeeplApi::new(ApiOptions::new_free(
        &std::env::var("DEEPL_API_KEY").unwrap(),
    ));
    let langs = api.languages(LanguageType::Target).await?;
    assert!(!langs.is_empty());
    Ok(())
}
