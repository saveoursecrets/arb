use anyhow::Result;
use arb_lib::{
    deepl::{ApiOptions, DeeplApi, Lang},
    ArbValue, Intl, Invalidation, TranslationOptions,
};
use serde_json::Value;

#[tokio::test]
pub async fn diff_cache() -> Result<()> {
    let api = DeeplApi::new(ApiOptions::new(&std::env::var("DEEPL_API_KEY").unwrap()));

    let index = "tests/fixtures/diff_update.yaml";
    let options = TranslationOptions {
        target_lang: Lang::Fr,
        dry_run: false,
        invalidation: Some(Invalidation::All),
        overrides: None,
        disable_cache: true,
    };
    let mut intl = Intl::new(index)?;
    let result = intl.translate(&api, options).await?;
    assert_eq!(1, result.length);

    let message = result.translated.lookup("message");
    assert!(message.is_some());

    let expected = Value::String("Bonjour le monde".to_owned());
    let expected_value: ArbValue<'_> = (&expected).into();
    assert_eq!(&expected_value, message.unwrap().value());

    Ok(())
}
