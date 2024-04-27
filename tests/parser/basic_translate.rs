use anyhow::Result;
use arb_lib::{
    deepl::{DeepLApi, Lang},
    translate, TranslationOptions,
};

#[tokio::test]
pub async fn basic_translate() -> Result<()> {
    let api = DeepLApi::with(&std::env::var("DEEPL_API_KEY").unwrap()).new();
    let index = "tests/fixtures/simple-arb-index.yaml";
    let options = TranslationOptions::new(index, Lang::FR);
    let translated = translate(api, options).await?;
    println!("{:#?}", translated);
    Ok(())
}