use anyhow::Result;
use arb_lib::{deepl::Lang, ArbIndex};

#[test]
pub fn diff_create() -> Result<()> {
    let index = ArbIndex::parse_yaml("tests/fixtures/diff_create.yaml", "app")?;

    let template = index.template_content()?;
    let french = index.load(Lang::Fr)?;

    let diff = template.diff(&french, index.cache().get_file(&Lang::Fr));
    assert!(diff.create.iter().any(|x| x == "fresh"));

    Ok(())
}

#[test]
pub fn diff_update() -> Result<()> {
    let index = ArbIndex::parse_yaml("tests/fixtures/diff_update.yaml", "app")?;

    let template = index.template_content()?;
    let french = index.load(Lang::Fr)?;

    let diff = template.diff(&french, index.cache().get_file(&Lang::Fr));
    assert!(diff.update.iter().any(|x| x == "message"));

    Ok(())
}

#[test]
pub fn diff_delete() -> Result<()> {
    let index = ArbIndex::parse_yaml("tests/fixtures/diff_delete.yaml", "app")?;

    let template = index.template_content()?;
    let french = index.load(Lang::Fr)?;

    let diff = template.diff(&french, index.cache().get_file(&Lang::Fr));
    assert!(diff.create.iter().any(|x| x == "message"));
    assert!(diff.delete.iter().any(|x| x == "obsolete"));

    Ok(())
}
