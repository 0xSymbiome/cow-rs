use std::{
    io::{self, Write},
    path::PathBuf,
    process::Command,
};

use anyhow::{Context, bail};

use crate::diagnostics::{Diagnostic, OutputMode};

const SHIPPED_CRATES: &[&str] = &[
    "cow-sdk-core",
    "cow-sdk-contracts",
    "cow-sdk-signing",
    "cow-sdk-orderbook",
    "cow-sdk-subgraph",
    "cow-sdk-app-data",
    "cow-sdk-trading",
    "cow-sdk-browser-wallet",
    "cow-sdk-transport-wasm",
    "cow-sdk-alloy-provider",
    "cow-sdk-alloy-signer",
    "cow-sdk-alloy",
    "cow-sdk",
];

const ALLOW_LIST: &[&str] = &["cow-sdk-alloy-provider", "cow-sdk-alloy"];

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlloyProviderEvaluation {
    Holds,
    Violated(String),
    Unexpected(String),
}

pub fn run(args: Args, output_mode: OutputMode) -> anyhow::Result<()> {
    let mut stdout = io::stdout().lock();
    run_with_writer(args, output_mode, &mut stdout)
}

pub fn run_with_writer(
    args: Args,
    output_mode: OutputMode,
    writer: &mut impl Write,
) -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&args.repo_root)
        .args(["tree", "--invert", "alloy-provider"]);
    for crate_name in SHIPPED_CRATES {
        cmd.args(["-p", crate_name]);
    }

    let output = cmd.output().context("failed to invoke cargo tree")?;
    let evaluation = evaluate_cargo_tree_output(
        output.status.success(),
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr),
    );

    match evaluation {
        AlloyProviderEvaluation::Holds => {
            Diagnostic::info("PM7000", "alloy-provider invariant holds")
                .emit(output_mode, writer)?;
            Ok(())
        }
        AlloyProviderEvaluation::Violated(detail) => {
            Diagnostic::error("PM7001", &detail).emit(output_mode, writer)?;
            bail!("{detail}")
        }
        AlloyProviderEvaluation::Unexpected(detail) => {
            Diagnostic::error("PM7002", &detail).emit(output_mode, writer)?;
            bail!("{detail}")
        }
    }
}

pub fn evaluate_cargo_tree_output(
    success: bool,
    stdout: &str,
    stderr: &str,
) -> AlloyProviderEvaluation {
    let combined = format!("{stdout}{stderr}").to_ascii_lowercase();
    if combined.contains("did not match any packages") {
        return AlloyProviderEvaluation::Holds;
    }
    if success && !stdout.trim().is_empty() {
        let violations = violating_crates(stdout, ALLOW_LIST);
        if violations.is_empty() {
            return AlloyProviderEvaluation::Holds;
        }
        return AlloyProviderEvaluation::Violated(format!(
            "alloy-provider is only allowed in cow-sdk-alloy-provider and cow-sdk-alloy; unexpected dependents: {}\n{stdout}",
            violations.join(", ")
        ));
    }
    AlloyProviderEvaluation::Unexpected(format!(
        "unexpected cargo tree output:\nstdout: {stdout}\nstderr: {stderr}"
    ))
}

fn violating_crates(stdout: &str, allow_list: &[&str]) -> Vec<String> {
    let mut violations = Vec::new();
    for line in stdout.lines() {
        let package = line
            .trim_start_matches(|c: char| {
                c.is_whitespace()
                    || matches!(c, '├' | '└' | '│' | '─' | ' ' | '`' | '-' | '+')
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
