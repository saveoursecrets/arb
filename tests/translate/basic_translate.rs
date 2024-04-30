use anyhow::Result;
use arb_lib::{
    deepl::{ApiOptions, DeeplApi, Lang},
    translate, ArbValue, TranslationOptions,
};
use serde_json::Value;

#[tokio::test]
pub async fn basic_translate() -> Result<()> {
    let api = DeeplApi::new(ApiOptions::new(&std::env::var("DEEPL_API_KEY").unwrap()));
    let index = "tests/fixtures/basic.yaml";
    let options = TranslationOptions::new(index, Lang::Fr);
    let result = translate(api, options).await?;

    // println!("{:#?}", result.translated);

    let hello_world = result.translated.lookup("helloWorld");
    let hello_name = result.translated.lookup("helloName");

    assert!(hello_world.is_some());
    assert!(hello_name.is_some());

    let expected = Value::String("Bonjour le monde".to_owned());
    let expected_value: ArbValue<'_> = (&expected).into();
    assert_eq!(&expected_value, hello_world.unwrap().value());

    let expected = Value::String("Bonjour {name}".to_owned());
    let expected_value: ArbValue<'_> = (&expected).into();
    assert_eq!(&expected_value, hello_name.unwrap().value());
    Ok(())
}
