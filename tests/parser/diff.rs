use anyhow::Result;
use arb_lib::{deepl::Lang, ArbIndex};

#[test]
pub fn diff() -> Result<()> {
    let index = ArbIndex::parse_yaml("tests/fixtures/diff.yaml", "app")?;

    let template = index.template_content()?;
    let french = index.load(Lang::Fr)?;

    println!("French: {:#?}", french);

    let diff = template.diff(&french);

    println!("{:#?}", diff);

    Ok(())
}
