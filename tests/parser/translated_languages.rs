use anyhow::Result;
use arb_lib::{deepl::Lang, ArbIndex};

#[test]
pub fn translated_languages() -> Result<()> {
    let index = ArbIndex::parse_yaml("tests/fixtures/translated_languages.yaml", "app")?;
    let translated = index.list_translated()?;
    assert!(translated.contains_key(&Lang::En));
    assert!(translated.contains_key(&Lang::Fr));
    Ok(())
}
