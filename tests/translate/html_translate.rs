use anyhow::Result;
use arb_lib::{
    deepl::{ApiOptions, DeeplApi, Lang},
    translate, ArbValue, TranslationOptions,
};
use serde_json::Value;

#[tokio::test]
pub async fn html_translate() -> Result<()> {
    let api = DeeplApi::new(ApiOptions::new(&std::env::var("DEEPL_API_KEY").unwrap()));
    let index = "tests/fixtures/html.yaml";
    let options = TranslationOptions::new(index, Lang::Fr);
    let result = translate(api, options).await?;

    // println!("{:#?}", result.translated);

    let links = result.translated.lookup("links");
    assert!(links.is_some());

    let expected = Value::String("En créant un compte, vous acceptez notre <a href=\"{privacyUrl}\">politique de confidentialité</a>, nos <a href=\"{termsUrl}\">conditions d'utilisation</a> et notre <a href=\"{acceptableUseUrl}\">politique d'utilisation acceptable</a>.".to_owned());
    let expected_value: ArbValue<'_> = (&expected).into();
    assert_eq!(&expected_value, links.unwrap().value());

    Ok(())
}
