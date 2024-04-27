use anyhow::Result;
use arb::ArbIndex;

#[test]
pub fn test_parse_index_with_template() -> Result<()> {
    let index = ArbIndex::parse_yaml("tests/fixtures/simple-arb-index.yaml")?;
    assert_eq!("simple-i10n", index.arb_dir());
    assert_eq!("app_en.arb", index.template_arb_file());

    let template = index.template_content()?;
    let entries = template.entries();
    assert!(!entries.is_empty());

    let value = template.lookup("helloWorld");
    assert!(value.is_some());
    assert!(value.as_ref().unwrap().is_translatable());
    assert_eq!("Hello world", value.unwrap().value().as_str().unwrap());

    let value = template.lookup("nonExistent");
    assert!(value.is_none());

    Ok(())
}
