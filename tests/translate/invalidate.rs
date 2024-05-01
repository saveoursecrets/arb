use anyhow::Result;
use arb_lib::{
    deepl::{ApiOptions, DeeplApi, Lang},
    Intl, Invalidation, TranslationOptions,
};

#[tokio::test]
pub async fn invalidate_all() -> Result<()> {
    let api = DeeplApi::new(ApiOptions::new(&std::env::var("DEEPL_API_KEY").unwrap()));

    let index = "tests/fixtures/invalidate.yaml";
    let options = TranslationOptions {
        target_lang: Lang::Fr,
        dry_run: false,
        invalidation: Some(Invalidation::All),
        overrides: None,
        disable_cache: false,
    };
    let mut intl = Intl::new(index)?;
    let result = intl.translate(&api, options).await?;
    assert_eq!(1, result.length);
    Ok(())
}

#[tokio::test]
pub async fn invalidate_keys() -> Result<()> {
    let api = DeeplApi::new(ApiOptions::new(&std::env::var("DEEPL_API_KEY").unwrap()));

    let index = "tests/fixtures/invalidate.yaml";
    let options = TranslationOptions {
        target_lang: Lang::Fr,
        dry_run: false,
        invalidation: Some(Invalidation::Keys(vec!["message".to_owned()])),
        overrides: None,
        disable_cache: false,
    };
    let mut intl = Intl::new(index)?;
    let result = intl.translate(&api, options).await?;
    assert_eq!(1, result.length);
    Ok(())
}
