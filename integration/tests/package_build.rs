use std::path::Path;

use integration::helpers::build_project_in_dir;

#[test]
fn paid_counter_packages_build() -> anyhow::Result<()> {
    let account = build_project_in_dir(Path::new("../contracts/paid-counter-account"), true)?;
    assert!(!account.mast.mast_forest().is_empty());

    let note = build_project_in_dir(Path::new("../contracts/paid-increment-note"), true)?;
    assert!(!note.mast.mast_forest().is_empty());

    Ok(())
}
