//! Keeps the published release-gate commands in agreement across the docs
//! and CI sites so drift is caught at the workspace level.
//!
//! Comparisons performed:
//!
//! 1. The `cargo tree --invert alloy-provider` `-p` package list must be
//!    identical across `docs/guides/release-checklist.md`, `docs/guides/verification.md`,
//!    `CONTRIBUTING.md`, and `docs/properties/index.md`.
//! 2. The `cargo audit` `--ignore RUSTSEC-…` token lists in
//!    `docs/guides/release-checklist.md` and `docs/guides/verification.md` must match the
//!    canonical advisory tolerance register in `.github/config/deny.toml`,
//!    and every canonical advisory must be documented in
//!    `docs/audit/dependency-gate-audit.md`.
//! 3. The property-citation registry and the audit-index review dates are
//!    re-validated through their own checks.

use std::{collections::BTreeSet, fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::docs::audit_index;
use crate::policy::check_property_citations;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

pub fn run(args: &Args) -> Result<()> {
    let root = &args.repo_root;
    let read = |rel: &str| -> Result<String> {
        fs::read_to_string(root.join(rel))
            .with_context(|| format!("required source file missing: {rel}"))
    };

    let release_checklist = read("docs/guides/release-checklist.md")?;
    let verification = read("docs/guides/verification.md")?;
    let deny_config = read(".github/config/deny.toml")?;
    let contributing = read("CONTRIBUTING.md")?;
    let properties = read("docs/properties/index.md")?;
    let dependency_gate_audit = read("docs/audit/dependency-gate-audit.md")?;

    // 1. cargo tree alloy-provider package-list agreement.
    let sites = [
        (
            "docs/guides/release-checklist.md",
            tree_packages(&release_checklist),
        ),
        ("docs/guides/verification.md", tree_packages(&verification)),
        ("CONTRIBUTING.md", tree_packages(&contributing)),
        ("docs/properties/index.md", tree_packages(&properties)),
    ];
    for (site, packages) in &sites {
        if packages.is_empty() {
            bail!("{site} does not declare the cargo tree alloy-provider package list");
        }
    }
    let (baseline_site, baseline) = &sites[0];
    for (site, packages) in &sites[1..] {
        if packages != baseline {
            bail!(
                "{baseline_site} and {site} disagree on the cargo tree alloy-provider package \
                 list:\n  {baseline_site}: {baseline:?}\n  {site}: {packages:?}"
            );
        }
    }

    // 2. cargo audit RUSTSEC ignore-token agreement.
    let checklist_tokens = audit_ignore_tokens(&release_checklist);
    let verification_tokens = audit_ignore_tokens(&verification);
    let canonical = canonical_audit_ignores(&deny_config)?;
    if checklist_tokens.is_empty() {
        bail!(
            "docs/guides/release-checklist.md does not declare the cargo audit ignore-token list"
        );
    }
    if verification_tokens.is_empty() {
        bail!("docs/guides/verification.md does not declare the cargo audit ignore-token list");
    }
    if canonical.is_empty() {
        bail!(
            ".github/config/deny.toml does not declare the canonical cargo audit ignore-token list"
        );
    }
    for (site, tokens) in [
        ("docs/guides/release-checklist.md", &checklist_tokens),
        ("docs/guides/verification.md", &verification_tokens),
    ] {
        if tokens != &canonical {
            bail!(
                "{site} and .github/config/deny.toml disagree on the cargo audit RUSTSEC \
                 ignore-token list:\n  {site}: {tokens:?}\n  canonical: {canonical:?}"
            );
        }
    }
    for advisory in &canonical {
        if !dependency_gate_audit.contains(advisory) {
            bail!(
                "docs/audit/dependency-gate-audit.md does not document cargo audit ignore token \
                 {advisory}"
            );
        }
    }

    // 3. Re-validate the property registry and the audit index.
    check_property_citations::run(&check_property_citations::Args {
        repo_root: root.clone(),
        properties: None,
    })?;
    audit_index::run(&audit_index::Args {
        repo_root: root.clone(),
    })?;

    println!("Release-gate commands agree across docs and CI.");
    Ok(())
}

/// Extracts the sorted `-p NAME` package set from every line bearing the
/// `cargo tree --invert alloy-provider` marker, following the workflow's
/// multi-line shell block until its `2>&1` redirect.
fn tree_packages(content: &str) -> BTreeSet<String> {
    let mut packages = BTreeSet::new();
    let mut capture = false;
    for line in content.lines() {
        if line.contains("cargo tree --invert alloy-provider") {
            capture = true;
        }
        if capture {
            collect_flag_values(line, "-p ", &mut packages);
            if line.contains("2>&1") || !line.trim_end().ends_with('\\') {
                capture = false;
            }
        }
    }
    packages
}

/// Extracts the sorted `--ignore RUSTSEC-####-####` token set from the first
/// `cargo audit --deny unsound` command, following backslash continuations.
fn audit_ignore_tokens(content: &str) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    let mut capture = false;
    for line in content.lines() {
        if !capture && line.contains("cargo audit --deny unsound") {
            capture = true;
        }
        if capture {
            for index in memchr_all(line, "--ignore RUSTSEC-") {
                let candidate = &line[index + "--ignore ".len()..];
                let token: String = candidate
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                    .collect();
                if is_rustsec_id(&token) {
                    tokens.insert(token);
                }
            }
            if !line.trim_end().ends_with('\\') {
                break;
            }
        }
    }
    tokens
}

fn collect_flag_values(line: &str, flag: &str, into: &mut BTreeSet<String>) {
    for index in memchr_all(line, flag) {
        let candidate = &line[index + flag.len()..];
        let value: String = candidate
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
            .collect();
        if value
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
        {
            into.insert(value);
        }
    }
}

fn memchr_all(haystack: &str, needle: &str) -> Vec<usize> {
    let mut indices = Vec::new();
    let mut start = 0;
    while let Some(found) = haystack[start..].find(needle) {
        indices.push(start + found);
        start += found + needle.len();
    }
    indices
}

fn is_rustsec_id(token: &str) -> bool {
    let bytes = token.as_bytes();
    bytes.len() == "RUSTSEC-0000-0000".len()
        && token.starts_with("RUSTSEC-")
        && bytes[8..12].iter().all(u8::is_ascii_digit)
        && bytes[12] == b'-'
        && bytes[13..17].iter().all(u8::is_ascii_digit)
}

#[derive(Debug, Deserialize)]
struct DenyConfig {
    advisories: Advisories,
}

#[derive(Debug, Deserialize)]
struct Advisories {
    #[serde(default)]
    ignore: Vec<IgnoreEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum IgnoreEntry {
    Detailed { id: String },
    Plain(String),
}

/// Reads the canonical advisory tolerance register from `deny.toml`.
fn canonical_audit_ignores(deny_toml: &str) -> Result<BTreeSet<String>> {
    let config: DenyConfig =
        toml::from_str(deny_toml).context("failed to parse .github/config/deny.toml")?;
    Ok(config
        .advisories
        .ignore
        .into_iter()
        .map(|entry| match entry {
            IgnoreEntry::Detailed { id } | IgnoreEntry::Plain(id) => id,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{audit_ignore_tokens, canonical_audit_ignores, tree_packages};

    #[test]
    fn package_lists_extract_from_single_and_multi_line_sites() {
        let markdown = "run `cargo tree --invert alloy-provider -p cow-sdk-core -p cow-sdk` now";
        let packages: Vec<_> = tree_packages(markdown).into_iter().collect();
        assert_eq!(packages, ["cow-sdk", "cow-sdk-core"]);

        let workflow = "          cargo tree --invert alloy-provider \\\n            -p cow-sdk-core \\\n            -p cow-sdk 2>&1\n          -p not-captured";
        let packages: Vec<_> = tree_packages(workflow).into_iter().collect();
        assert_eq!(packages, ["cow-sdk", "cow-sdk-core"]);
    }

    #[test]
    fn audit_tokens_extract_and_canonical_register_parses() {
        let block = "cargo audit --deny unsound \\\n  --ignore RUSTSEC-2024-0436 \\\n  --ignore RUSTSEC-2024-0388\nlater --ignore RUSTSEC-9999-9999";
        let tokens: Vec<_> = audit_ignore_tokens(block).into_iter().collect();
        assert_eq!(tokens, ["RUSTSEC-2024-0388", "RUSTSEC-2024-0436"]);

        let deny =
            "[advisories]\nignore = [\n  { id = \"RUSTSEC-2024-0436\", reason = \"x\" },\n]\n";
        let canonical: Vec<_> = canonical_audit_ignores(deny)
            .expect("deny config parses")
            .into_iter()
            .collect();
        assert_eq!(canonical, ["RUSTSEC-2024-0436"]);
    }
}
