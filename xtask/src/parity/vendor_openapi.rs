//! Vendors the lock-pinned services `OpenAPI` document.
//!
//! Zero-argument by design: the services checkout is materialized (or
//! re-detached) at the pinned commit under the shared upstream root, so
//! `cargo parity-vendor-openapi` is the whole refresh step after a pin bump.
//! The stamped output is what `validate` and `openapi-coverage` gate on.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Args;

use crate::parity::{
    SERVICES_OPENAPI_PATH, VENDORED_STAMP_PREFIX, load_source_lock, repository_entry, sync,
    validate_repository_root, vendored_openapi_path,
};

#[derive(Debug, Args)]
pub struct VendorOpenApiArgs {
    #[arg(long, default_value = crate::parity::DEFAULT_SOURCE_LOCK)]
    source_lock: PathBuf,
    /// Root containing the upstream checkouts (`<root>/services`); the
    /// services checkout is cloned and pinned on demand.
    #[arg(long, env = "XTASK_UPSTREAM_ROOT", default_value = sync::DEFAULT_UPSTREAM_ROOT)]
    root: PathBuf,
    /// Output path (default: `openapi/services-orderbook.yml` next to the lock).
    #[arg(long)]
    output: Option<PathBuf>,
}

pub fn run(args: &VendorOpenApiArgs) -> Result<()> {
    let lock = load_source_lock(&args.source_lock)?;
    let services = repository_entry(&lock, "services")?;

    let checkout = args.root.join(&services.id);
    sync::ensure_checkout(services, &checkout, false)?;
    sync::fetch_commit(&checkout, &services.commit)
        .with_context(|| format!("pin {} unreachable for services", services.commit))?;
    sync::checkout_detached(&checkout, &services.commit, false)?;
    validate_repository_root(services, &checkout)?;

    let source = checkout.join(SERVICES_OPENAPI_PATH);
    let raw = fs::read_to_string(&source)
        .with_context(|| format!("failed to read {}", source.display()))?;

    let output = args
        .output
        .clone()
        .unwrap_or_else(|| vendored_openapi_path(&args.source_lock));
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let stamped = format!(
        "{VENDORED_STAMP_PREFIX}{}\n# Path: {SERVICES_OPENAPI_PATH}\n# DO NOT EDIT - regenerate via `cargo parity-vendor-openapi`.\n{raw}",
        services.commit
    );
    fs::write(&output, stamped).with_context(|| format!("failed to write {}", output.display()))?;

    println!(
        "vendored services OpenAPI at commit {} into {}",
        services.commit,
        output.display()
    );
    Ok(())
}
