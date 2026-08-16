use std::path::Path;

use anyhow::{bail, Context, Result};
use cargo_miden::run;
use miden_client::{account::StorageSlotName, Felt, Word};
use miden_mast_package::Package;

pub const COUNTER_STORAGE_KEY: Word =
    Word::new([Felt::ZERO, Felt::ZERO, Felt::ZERO, Felt::ONE]);

pub fn counter_storage_slot() -> Result<StorageSlotName> {
    StorageSlotName::new("paid_counter_account::paid_counter::count_map")
        .context("invalid paid counter storage slot name")
}

pub fn build_project_in_dir(dir: &Path, release: bool) -> Result<Package> {
    let profile = if release { "--release" } else { "--debug" };
    let manifest_path = dir.join("Cargo.toml");
    let manifest_arg = manifest_path.to_string_lossy();

    let args = vec![
        "cargo",
        "miden",
        "build",
        profile,
        "--manifest-path",
        &manifest_arg,
    ];

    let output = run(args.into_iter().map(String::from))
        .context("failed to compile Miden project")?
        .context("cargo miden build returned no output")?;

    let artifact_path = match output {
        cargo_miden::CommandOutput::BuildCommandOutput { output } => output
            .into_iter()
            .next()
            .context("cargo miden build produced no artifact")?,
        other => bail!("expected build output, got {:?}", other),
    };

    let package_bytes = std::fs::read(&artifact_path)
        .with_context(|| format!("failed to read {}", artifact_path.display()))?;

    Package::read_from_bytes(&package_bytes).context("failed to deserialize Miden package")
}
