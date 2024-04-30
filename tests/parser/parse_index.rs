use anyhow::Result;
use arb_lib::{ArbIndex, ArbKey};

#[test]
pub fn parse_index_with_template() -> Result<()> {
    let index = ArbIndex::parse_yaml("tests/fixtures/basic.yaml", "app")?;
    assert_eq!("basic", index.arb_dir());
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

    let key_name = ArbKey::new("helloName");
    let placeholders = template.placeholders(&key_name)?;
    assert_eq!(placeholders.unwrap().to_vec(), vec!["name"]);

    Ok(())
}
