use anyhow::Result;
use arb_lib::{deepl::Lang, Intl};

#[test]
pub fn diff_create() -> Result<()> {
    let index = Intl::new("tests/fixtures/diff_create.yaml")?;

    let template = index.template_content()?;
    let french = index.load(Lang::Fr)?;

    let diff = template.diff(&french, index.cache().get_file(&Lang::Fr));
    assert!(diff.create.iter().any(|x| x == "fresh"));

    Ok(())
}

#[test]
pub fn diff_update() -> Result<()> {
    let index = Intl::new("tests/fixtures/diff_update.yaml")?;

    let template = index.template_content()?;
    let french = index.load(Lang::Fr)?;

    let diff = template.diff(&french, index.cache().get_file(&Lang::Fr));
    assert!(diff.update.iter().any(|x| x == "message"));

    Ok(())
}

#[test]
pub fn diff_delete() -> Result<()> {
    let index = Intl::new("tests/fixtures/diff_delete.yaml")?;

    let template = index.template_content()?;
    let french = index.load(Lang::Fr)?;

    let diff = template.diff(&french, index.cache().get_file(&Lang::Fr));
    assert!(diff.create.iter().any(|x| x == "message"));
    assert!(diff.delete.iter().any(|x| x == "obsolete"));

    Ok(())
}
