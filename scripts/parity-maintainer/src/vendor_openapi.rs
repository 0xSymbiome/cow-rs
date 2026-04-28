use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::{SecondsFormat, Utc};
use clap::Args;

use crate::{load_source_lock, repository_entry, validate_repository_root};

const SERVICES_OPENAPI_PATH: &str = "crates/orderbook/openapi.yml";
const DEFAULT_OUTPUT: &str = "parity/openapi/services-orderbook.yml";

#[derive(Debug, Args)]
pub(crate) struct VendorOpenApiArgs {
    #[arg(long, default_value = crate::DEFAULT_SOURCE_LOCK)]
    source_lock: PathBuf,
    #[arg(long)]
    services_root: PathBuf,
    #[arg(long, default_value = DEFAULT_OUTPUT)]
    output: PathBuf,
}

pub(crate) fn run(args: VendorOpenApiArgs) -> Result<()> {
    vendor_openapi(&args.source_lock, &args.services_root, &args.output)
}

fn vendor_openapi(source_lock: &Path, services_root: &Path, output: &Path) -> Result<()> {
    let lock = load_source_lock(source_lock)?;
    let services_repo = repository_entry(&lock, "services")?;
    validate_repository_root(services_repo, services_root)?;

    let source = services_root.join(SERVICES_OPENAPI_PATH);
    let raw = fs::read_to_string(&source)
        .with_context(|| format!("failed to read {}", source.display()))?;

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let generated = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let stamped = format!(
        "# Vendored from cowprotocol/services @ {}\n# Path: {}\n# Generated: {}\n# DO NOT EDIT - regenerate via `parity-maintainer vendor-openapi`.\n{}",
        services_repo.commit, SERVICES_OPENAPI_PATH, generated, raw
    );
    fs::write(output, stamped).with_context(|| format!("failed to write {}", output.display()))?;

    println!(
        "vendored services OpenAPI from {} at commit {} into {}",
        source.display(),
        services_repo.commit,
        output.display()
    );
    Ok(())
}
