use anyhow::Result;
use arb_lib::{
    deepl::{ApiOptions, DeeplApi, Lang},
    translate, TranslationOptions,
};

#[tokio::test]
pub async fn html_translate() -> Result<()> {
    let api = DeeplApi::new(ApiOptions::new_free(
        &std::env::var("DEEPL_API_KEY").unwrap(),
    ));
    let index = "tests/fixtures/html-index.yaml";
    let options = TranslationOptions::new(index, Lang::Fr);
    let translated = translate(api, options).await?;
    println!("{:#?}", translated);
    Ok(())
}
