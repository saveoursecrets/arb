use anyhow::Result;
use arb::parser::ArbIndex;

#[test]
pub fn test_parse_index_with_template() -> Result<()> {
    let index = ArbIndex::parse_yaml("tests/fixtures/arb-index.yaml")?;
    assert_eq!("i10n", index.arb_dir());
    assert_eq!("app_en.arb", index.template_arb_file());

    let template = index.template_content()?;

    println!("{:#?}", template);

    Ok(())
}
