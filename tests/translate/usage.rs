use anyhow::Result;
use arb_lib::deepl::{ApiOptions, DeeplApi};

#[tokio::test]
pub async fn usage() -> Result<()> {
    let api = DeeplApi::new(ApiOptions::new_free(
        &std::env::var("DEEPL_API_KEY").unwrap(),
    ));
    let usage = api.usage().await?;
    assert!(usage.character_limit > 0);
    Ok(())
}
