use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, bail};

use crate::diagnostics::{Diagnostic, OutputMode};

pub const EXAMPLE_RUNNER_DIAGNOSTIC_CODE: &str = "PM9000";

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Pass --locked through to every cargo run invocation.
    #[arg(long)]
    pub locked: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeterministicExample {
    WorkspacePackage {
        package: &'static str,
        example: &'static str,
    },
    Manifest {
        manifest_path: &'static str,
        example: &'static str,
    },
}

pub const DETERMINISTIC_EXAMPLES: &[DeterministicExample] = &[
    DeterministicExample::WorkspacePackage {
        package: "cow-sdk",
        example: "wasm_smoke",
    },
    DeterministicExample::WorkspacePackage {
        package: "cow-sdk-trading",
        example: "signed_order_end_to_end",
    },
    DeterministicExample::WorkspacePackage {
        package: "cow-sdk-trading",
        example: "typestate_builder_example",
    },
    DeterministicExample::WorkspacePackage {
        package: "cow-sdk-orderbook",
        example: "paginated_orders_fetch",
    },
    DeterministicExample::WorkspacePackage {
        package: "cow-sdk-subgraph",
        example: "typed_query_with_escape_hatch",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "sdk_surface_report",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "app_data_roundtrip",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "signing_roundtrip",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "quote_only_simulation",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "limit_order_simulation",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "order_lifecycle_simulation",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "trading_sdk_simulation",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "ethflow_transaction_simulation",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "onchain_order_actions_simulation",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "orderbook_transport_roundtrip",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "subgraph_query_roundtrip",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "simplest_swap_quickstart",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "error_classification_simulation",
    },
    DeterministicExample::Manifest {
        manifest_path: "examples/native/Cargo.toml",
        example: "order_list_history_simulation",
    },
];

pub fn run(args: Args, output_mode: OutputMode) -> anyhow::Result<()> {
    let mut stdout = io::stdout().lock();
    run_with_writer(args, output_mode, &mut stdout)
}

pub fn run_with_writer(
    args: Args,
    output_mode: OutputMode,
    writer: &mut impl Write,
) -> anyhow::Result<()> {
    for example in DETERMINISTIC_EXAMPLES {
        Diagnostic::info(
            EXAMPLE_RUNNER_DIAGNOSTIC_CODE,
            format!("running deterministic example {}", example.label()),
        )
        .emit(output_mode, writer)?;

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

    Diagnostic::info(
        EXAMPLE_RUNNER_DIAGNOSTIC_CODE,
        format!(
            "ran {} deterministic example binary/binaries",
            DETERMINISTIC_EXAMPLES.len()
        ),
    )
    .emit(output_mode, writer)?;
    Ok(())
}

impl DeterministicExample {
    pub const fn label(self) -> &'static str {
        match self {
            Self::WorkspacePackage { example, .. } | Self::Manifest { example, .. } => example,
        }
    }

    fn command(self, repo_root: &Path, locked: bool) -> Command {
        let mut command = Command::new("cargo");
        command.current_dir(repo_root).arg("run").arg("--quiet");
        if locked {
            command.arg("--locked");
        }
        match self {
            Self::WorkspacePackage { package, example } => {
                command.arg("-p").arg(package).arg("--example").arg(example);
            }
            Self::Manifest {
                manifest_path,
                example,
            } => {
                command
                    .arg("--manifest-path")
                    .arg(manifest_path)
                    .arg("--example")
                    .arg(example);
            }
        }
        command
    }
}

#[cfg(test)]
mod tests {
    use super::{DETERMINISTIC_EXAMPLES, DeterministicExample};

    #[test]
    fn deterministic_runner_excludes_live_examples() {
        let labels: Vec<&str> = DETERMINISTIC_EXAMPLES
            .iter()
            .copied()
            .map(DeterministicExample::label)
            .collect();

        assert!(!labels.contains(&"orderbook_live_probe"));
        assert!(!labels.contains(&"subgraph_live_query"));
    }

    #[test]
    fn deterministic_runner_covers_flagship_trading_example() {
        let labels: Vec<&str> = DETERMINISTIC_EXAMPLES
            .iter()
            .copied()
            .map(DeterministicExample::label)
            .collect();

        assert!(labels.contains(&"signed_order_end_to_end"));
        assert!(labels.contains(&"trading_sdk_simulation"));
    }
}
