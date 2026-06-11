//! Native-adapter dependency-isolation gate shared by the alloy provider and
//! signer invariants.
//!
//! A heavyweight native alloy crate (`alloy-provider`, `alloy-signer-local`)
//! must stay confined to the native adapter crates allowed to pull it. Each
//! invariant inspects the inverse dependency tree of that crate across every
//! published workspace crate and fails if a crate outside the allow-list
//! appears. The published-crate roster is read from the workspace manifest
//! (see [`workspace::shipped_crates`]), so a newly added crate is covered
//! automatically.
//!
//! The invariant protects the *shipped* dependency graph, so the inverse tree
//! follows non-dev edges only (`--edges no-dev`): a dev-dependency is stripped
//! from a published crate's dependency graph and cannot leak the heavyweight
//! native crate to a consumer. A test that exercises the real signer or
//! provider (for example to produce a recoverable signature) therefore stays
//! allowed, while any normal- or build-dependency leak is still caught.

use std::{path::PathBuf, process::Command};

use anyhow::{Context, bail};

use crate::policy::workspace;

/// One native-adapter isolation invariant.
pub struct Invariant {
    /// crates.io package whose inverse dependency tree is inspected.
    pub crate_name: &'static str,
    /// Published workspace crates permitted to depend on `crate_name`.
    pub allow_list: &'static [&'static str],
}

pub const ALLOY_PROVIDER: Invariant = Invariant {
    crate_name: "alloy-provider",
    allow_list: &["cow-sdk-alloy-provider", "cow-sdk-alloy"],
};

pub const ALLOY_SIGNER: Invariant = Invariant {
    crate_name: "alloy-signer-local",
    allow_list: &["cow-sdk-alloy-signer", "cow-sdk-alloy"],
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Evaluation {
    Holds,
    Violated(String),
    Unexpected(String),
}

pub fn run_alloy_provider_default() -> anyhow::Result<()> {
    run(
        &ALLOY_PROVIDER,
        &Args {
            repo_root: PathBuf::from("."),
        },
    )
}

pub fn run_alloy_signer_default() -> anyhow::Result<()> {
    run(
        &ALLOY_SIGNER,
        &Args {
            repo_root: PathBuf::from("."),
        },
    )
}

pub fn run(invariant: &Invariant, args: &Args) -> anyhow::Result<()> {
    let shipped = workspace::shipped_crates(&args.repo_root)?;
    let mut command = Command::new("cargo");
    command
        .current_dir(&args.repo_root)
        // `--edges no-dev`: the invariant guards the shipped graph, and a
        // dev-dependency never ships, so a test that pulls the real signer or
        // provider does not leak it to consumers. Normal/build leaks still fail.
        .args(["tree", "--edges", "no-dev", "--invert", invariant.crate_name]);
    for crate_name in &shipped {
        command.args(["-p", crate_name]);
    }

    let output = command.output().context("failed to invoke cargo tree")?;
    let evaluation = evaluate_cargo_tree_output(
        invariant,
        output.status.success(),
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    );

    match evaluation {
        Evaluation::Holds => {
            println!("{} invariant holds", invariant.crate_name);
            Ok(())
        }
        Evaluation::Violated(detail) | Evaluation::Unexpected(detail) => {
            eprintln!("error: {detail}");
            bail!("{detail}")
        }
    }
}

pub fn evaluate_cargo_tree_output(
    invariant: &Invariant,
    success: bool,
    stdout: &str,
    stderr: &str,
) -> Evaluation {
    let combined = format!("{stdout}{stderr}").to_ascii_lowercase();
    if combined.contains("did not match any packages") {
        return Evaluation::Holds;
    }
    if success && !stdout.trim().is_empty() {
        let violations = violating_crates(stdout, invariant.allow_list);
        if violations.is_empty() {
            return Evaluation::Holds;
        }
        return Evaluation::Violated(format!(
            "{} is only allowed in {}; unexpected dependents: {}\n{stdout}",
            invariant.crate_name,
            invariant.allow_list.join(" and "),
            violations.join(", ")
        ));
    }
    Evaluation::Unexpected(format!(
        "unexpected cargo tree output:\nstdout: {stdout}\nstderr: {stderr}"
    ))
}

fn violating_crates(stdout: &str, allow_list: &[&str]) -> Vec<String> {
    let mut violations = Vec::new();
    for line in stdout.lines() {
        let package = line
            .trim_start_matches(|c: char| {
                c.is_whitespace() || matches!(c, '├' | '└' | '│' | '─' | ' ' | '`' | '-' | '+')
            })
            .split_whitespace()
            .next()
            .unwrap_or_default();
        if package.starts_with("cow-sdk")
            && !allow_list.contains(&package)
            && !violations.iter().any(|seen| seen == package)
        {
            violations.push(package.to_owned());
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::{ALLOY_PROVIDER, ALLOY_SIGNER, Evaluation, evaluate_cargo_tree_output};

    #[test]
    fn missing_package_is_treated_as_holding() {
        let evaluation = evaluate_cargo_tree_output(
            &ALLOY_PROVIDER,
            false,
            "",
            "error: package ID specification `alloy-provider` did not match any packages",
        );
        assert_eq!(evaluation, Evaluation::Holds);
    }

    #[test]
    fn an_unexpected_dependent_violates() {
        let tree = "alloy-signer-local v1.0.0\n└── cow-sdk-orderbook v0.1.0\n";
        let evaluation = evaluate_cargo_tree_output(&ALLOY_SIGNER, true, tree, "");
        let Evaluation::Violated(detail) = evaluation else {
            panic!("expected a violation, got {evaluation:?}");
        };
        assert!(detail.contains("cow-sdk-orderbook"));
        assert!(detail.contains("cow-sdk-alloy-signer and cow-sdk-alloy"));
    }

    #[test]
    fn only_allow_listed_dependents_hold() {
        let tree = "alloy-provider v1.0.0\n└── cow-sdk-alloy-provider v0.1.0\n    └── cow-sdk-alloy v0.1.0\n";
        let evaluation = evaluate_cargo_tree_output(&ALLOY_PROVIDER, true, tree, "");
        assert_eq!(evaluation, Evaluation::Holds);
    }
}
