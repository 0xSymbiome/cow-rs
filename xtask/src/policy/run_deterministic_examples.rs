use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, bail};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Pass --locked through to every cargo run invocation.
    #[arg(long)]
    pub locked: bool,
}

/// Every deterministic example lives in the one native cookbook manifest.
const NATIVE_EXAMPLES_MANIFEST: &str = "examples/native/Cargo.toml";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeterministicExample {
    example: &'static str,
    /// Crate features the example needs to compile (empty for the
    /// default-feature scenarios).
    features: &'static [&'static str],
}

/// The deterministic, non-live native example binaries, each paired with the
/// features it needs to compile.
///
/// This is the full non-live example surface — every `[[example]]` in the
/// cookbook except the two opt-in live probes (`orderbook_live`,
/// `subgraph_live`). `deterministic_runner_covers_every_non_live_example`
/// enforces that equality, so a newly added example cannot silently escape the
/// smoke run and falsify the "every deterministic non-live binary" claim the
/// example docs make.
pub const DETERMINISTIC_EXAMPLES: &[DeterministicExample] = &[
    DeterministicExample::new("facade_surface", &[]),
    DeterministicExample::new("app_data", &[]),
    DeterministicExample::new("sign_order", &[]),
    DeterministicExample::new("quote", &[]),
    DeterministicExample::new("cancel_in_flight", &[]),
    DeterministicExample::new("limit_order", &[]),
    DeterministicExample::new("order_lifecycle", &[]),
    DeterministicExample::new("trading_full_cycle", &[]),
    DeterministicExample::new("ethflow", &[]),
    DeterministicExample::new("onchain_actions", &[]),
    DeterministicExample::new("orderbook_transport", &[]),
    DeterministicExample::new("subgraph_query", &[]),
    DeterministicExample::new("swap_quickstart", &[]),
    DeterministicExample::new("error_classification", &[]),
    DeterministicExample::new("order_history", &[]),
    DeterministicExample::new("receipt_lifecycle", &[]),
    DeterministicExample::new("slippage_suggester", &[]),
    DeterministicExample::new("eip1271_signer", &[]),
    DeterministicExample::new("receiver_redirect", &[]),
    DeterministicExample::new("ethflow_checker", &[]),
    DeterministicExample::new("twap_order", &[]),
    DeterministicExample::new("alloy_quickstart", &["alloy"]),
    DeterministicExample::new("alloy_provider", &["alloy-provider"]),
    DeterministicExample::new("token_balance", &["alloy-provider"]),
    DeterministicExample::new("alloy_signer", &["alloy-signer"]),
    DeterministicExample::new("alloy_custom_traits", &["alloy"]),
    DeterministicExample::new("alloy_trading_full_flow", &["alloy"]),
    DeterministicExample::new("transaction_lifecycle", &["alloy"]),
];

pub fn run(args: &Args) -> anyhow::Result<()> {
    for example in DETERMINISTIC_EXAMPLES {
        println!("running deterministic example {}", example.label());

        let status = example
            .command(&args.repo_root, args.locked)
            .status()
            .with_context(|| format!("failed to spawn cargo for {}", example.label()))?;
        if !status.success() {
            bail!(
                "deterministic example {} failed with status {status}",
                example.label()
            );
        }
    }

    println!(
        "ran {} deterministic example binary/binaries",
        DETERMINISTIC_EXAMPLES.len()
    );
    Ok(())
}

impl DeterministicExample {
    const fn new(example: &'static str, features: &'static [&'static str]) -> Self {
        Self { example, features }
    }

    pub const fn label(self) -> &'static str {
        self.example
    }

    fn command(self, repo_root: &Path, locked: bool) -> Command {
        let mut command = Command::new("cargo");
        command.current_dir(repo_root).arg("run").arg("--quiet");
        if locked {
            command.arg("--locked");
        }
        command
            .arg("--manifest-path")
            .arg(NATIVE_EXAMPLES_MANIFEST)
            .arg("--example")
            .arg(self.example);
        if !self.features.is_empty() {
            command.arg("--features").arg(self.features.join(","));
        }
        command
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{DETERMINISTIC_EXAMPLES, DeterministicExample};

    fn runner_labels() -> Vec<&'static str> {
        DETERMINISTIC_EXAMPLES
            .iter()
            .copied()
            .map(DeterministicExample::label)
            .collect()
    }

    #[test]
    fn deterministic_runner_excludes_live_examples() {
        let labels = runner_labels();

        assert!(!labels.contains(&"orderbook_live"));
        assert!(!labels.contains(&"subgraph_live"));
    }

    #[test]
    fn deterministic_runner_covers_flagship_trading_example() {
        let labels = runner_labels();

        assert!(labels.contains(&"swap_quickstart"));
        assert!(labels.contains(&"trading_full_cycle"));
    }

    #[test]
    fn deterministic_runner_covers_every_non_live_example() {
        // The smoke runner claims to cover "every deterministic non-live binary"
        // (examples/README.md, docs/guides/examples.md). Enforce it mechanically: the
        // run set must equal every declared `[[example]]` minus the two opt-in
        // live probes, so a newly added example cannot silently fall out of the
        // smoke command and falsify that claim.
        const LIVE: &[&str] = &["orderbook_live", "subgraph_live"];

        let manifest = include_str!("../../../examples/native/Cargo.toml");
        let declared: BTreeSet<&str> = manifest
            .lines()
            .filter_map(|line| line.trim().strip_prefix("name = \""))
            .filter_map(|rest| rest.strip_suffix('"'))
            .filter(|name| *name != "cow-sdk-examples-native")
            .collect();
        let expected: BTreeSet<&str> = declared
            .iter()
            .copied()
            .filter(|name| !LIVE.contains(name))
            .collect();
        let actual: BTreeSet<&str> = runner_labels().into_iter().collect();

        assert_eq!(
            actual, expected,
            "deterministic runner must cover every non-live native example"
        );
    }
}
