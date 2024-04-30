use anyhow::Result;
use arb_lib::{
    deepl::{ApiOptions, DeeplApi, Lang},
    translate, ArbValue, Invalidation, TranslationOptions,
};
use serde_json::Value;
use std::path::PathBuf;

#[tokio::test]
pub async fn diff_cache() -> Result<()> {
    let api = DeeplApi::new(ApiOptions::new_free(
        &std::env::var("DEEPL_API_KEY").unwrap(),
    ));

    let index = "tests/fixtures/diff_update.yaml";
    let options = TranslationOptions {
        index_file: PathBuf::from(index),
        target_lang: Lang::Fr,
        dry_run: false,
        name_prefix: "app".to_owned(),
        invalidation: Some(Invalidation::All),
        overrides: None,
        disable_cache: true,
    };
    let result = translate(api, options).await?;
    assert_eq!(1, result.length);

    let message = result.translated.lookup("message");
    assert!(message.is_some());

    let expected = Value::String("Bonjour le monde".to_owned());
    let expected_value: ArbValue<'_> = (&expected).into();
    assert_eq!(&expected_value, message.unwrap().value());

    Ok(())
}
