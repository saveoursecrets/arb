use anyhow::Result;
use arb_lib::{
    deepl::{ApiOptions, DeeplApi, Lang},
    translate, Invalidation, TranslationOptions,
};
use std::path::PathBuf;

#[tokio::test]
pub async fn invalidate_all() -> Result<()> {
    let api = DeeplApi::new(ApiOptions::new_free(
        &std::env::var("DEEPL_API_KEY").unwrap(),
    ));

    let index = "tests/fixtures/invalidate.yaml";
    let options = TranslationOptions {
        index_file: PathBuf::from(index),
        target_lang: Lang::Fr,
        dry_run: false,
        name_prefix: "app".to_owned(),
        invalidation: Some(Invalidation::All),
        overrides: None,
    };
    let result = translate(api, options).await?;
    assert_eq!(1, result.length);
    Ok(())
}

#[tokio::test]
pub async fn invalidate_keys() -> Result<()> {
    let api = DeeplApi::new(ApiOptions::new_free(
        &std::env::var("DEEPL_API_KEY").unwrap(),
    ));

    let index = "tests/fixtures/invalidate.yaml";
    let options = TranslationOptions {
        index_file: PathBuf::from(index),
        target_lang: Lang::Fr,
        dry_run: false,
        name_prefix: "app".to_owned(),
        invalidation: Some(Invalidation::Keys(vec!["message".to_owned()])),
        overrides: None,
    };
    let result = translate(api, options).await?;
    assert_eq!(1, result.length);
    Ok(())
}
