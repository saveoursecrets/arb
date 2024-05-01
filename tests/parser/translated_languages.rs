use anyhow::Result;
use arb_lib::{deepl::Lang, Intl};

#[test]
pub fn translated_languages() -> Result<()> {
    let index = Intl::new("tests/fixtures/translated_languages.yaml")?;
    let translated = index.list_translated()?;
    assert!(translated.contains_key(&Lang::En));
    assert!(translated.contains_key(&Lang::Fr));
    Ok(())
}
