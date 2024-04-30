use anyhow::Result;
use arb_lib::{
    deepl::{ApiOptions, DeeplApi, Lang},
    translate, ArbEntry, ArbFile, ArbValue, TranslationOptions,
};
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf};

#[tokio::test]
pub async fn overrides() -> Result<()> {
    let api = DeeplApi::new(ApiOptions::new(&std::env::var("DEEPL_API_KEY").unwrap()));

    let mut overrides_file = ArbFile::default();
    let value = Value::String("Salut".to_string());
    overrides_file.insert_entry(ArbEntry::new("message", &value));

    let mut overrides = HashMap::new();
    overrides.insert(Lang::Fr, overrides_file);

    let index = "tests/fixtures/invalidate.yaml";
    let options = TranslationOptions {
        index_file: PathBuf::from(index),
        target_lang: Lang::Fr,
        dry_run: false,
        name_prefix: "app".to_owned(),
        invalidation: None,
        overrides: Some(overrides),
        disable_cache: false,
    };

    let result = translate(api, options).await?;
    // Not translated because overriden
    assert_eq!(0, result.length);

    let message = result.translated.lookup("message");
    assert!(message.is_some());

    let expected = Value::String("Salut".to_owned());
    let expected_value: ArbValue<'_> = (&expected).into();
    assert_eq!(&expected_value, message.unwrap().value());

    Ok(())
}
