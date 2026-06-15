//! Verifies each consumer crate renders its README on docs.rs.
//!
//! The former docs-quality lane built nightly rustdoc and scraped each crate's
//! HTML to confirm the README heading rendered. The invariant it was really
//! protecting is that the crate's `lib.rs` includes the README into the docs.rs
//! build via `#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]`
//! (or an unconditional `#![doc = ...]`). This check asserts that at the source
//! level, without rendering rustdoc.

use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::policy::workspace;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

/// Consumer crate directories whose README must render on docs.rs. The thin
/// native alloy adapter crates deliberately include their README under
/// `doctest` only, so they stay out of scope (as in the former docs render
/// check).
const README_RENDERING_CRATES: &[&str] = &[
    "core",
    "contracts",
    "signing",
    "app-data",
    "orderbook",
    "trading",
    "subgraph",
    "browser-wallet",
    "sdk",
];

const README_DOC: &str = r#"doc = include_str!("../README.md")"#;

pub fn run_default() -> Result<()> {
    run(&Args {
        repo_root: PathBuf::from("."),
    })
}

pub fn run(args: &Args) -> Result<()> {
    let mut errors = Vec::new();
    for crate_dir in README_RENDERING_CRATES {
        let lib = args
            .repo_root
            .join("crates")
            .join(crate_dir)
            .join("src/lib.rs");
        let text = workspace::read_to_string(&lib)?;
        if !renders_readme_on_docsrs(&text) {
            errors.push(format!(
                "crates/{crate_dir}/src/lib.rs must render its README on docs.rs via `#![cfg_attr(any(doctest, docsrs), doc = include_str!(\"../README.md\"))]`"
            ));
        }
    }

    if errors.is_empty() {
        println!(
            "README docs.rs inclusion holds across {} crate(s)",
            README_RENDERING_CRATES.len()
        );
        return Ok(());
    }
    for error in &errors {
        eprintln!("error: {error}");
    }
    bail!("README inclusion has {} error(s)", errors.len())
}

/// True when `lib.rs` includes the README in a form that is active on docs.rs:
/// an unconditional `#![doc = include_str!(...)]` or a `cfg_attr` whose
/// condition includes `docsrs`. A `doctest`-only inclusion does not render.
fn renders_readme_on_docsrs(lib_rs: &str) -> bool {
    lib_rs.lines().any(|line| {
        line.contains(README_DOC) && (line.contains("docsrs") || !line.contains("cfg_attr"))
    })
}

#[cfg(test)]
mod tests {
    use super::renders_readme_on_docsrs;

    #[test]
    fn docsrs_and_unconditional_render_but_doctest_only_does_not() {
        assert!(renders_readme_on_docsrs(
            r#"#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]"#
        ));
        assert!(renders_readme_on_docsrs(
            r#"#![doc = include_str!("../README.md")]"#
        ));
        assert!(!renders_readme_on_docsrs(
            r#"#![cfg_attr(doctest, doc = include_str!("../README.md"))]"#
        ));
        assert!(!renders_readme_on_docsrs(
            "//! crate docs without a README include"
        ));
    }
}
