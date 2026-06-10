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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeterministicExample {
    manifest_path: &'static str,
    example: &'static str,
}

pub const DETERMINISTIC_EXAMPLES: &[DeterministicExample] = &[
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "facade_surface",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "app_data",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "sign_order",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "quote",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "limit_order",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "order_lifecycle",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "trading_full_cycle",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "ethflow",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "onchain_actions",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "orderbook_transport",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "subgraph_query",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "swap_quickstart",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "error_classification",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "order_history",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "receipt_lifecycle",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "slippage_suggester",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "eip1271_signer",
    },
    DeterministicExample {
        manifest_path: "examples/native/Cargo.toml",
        example: "ethflow_checker",
    },
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
            .arg(self.manifest_path)
            .arg("--example")
            .arg(self.example);
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

        assert!(!labels.contains(&"orderbook_live"));
        assert!(!labels.contains(&"subgraph_live"));
    }

    #[test]
    fn deterministic_runner_covers_flagship_trading_example() {
        let labels: Vec<&str> = DETERMINISTIC_EXAMPLES
            .iter()
            .copied()
            .map(DeterministicExample::label)
            .collect();

        assert!(labels.contains(&"swap_quickstart"));
        assert!(labels.contains(&"trading_full_cycle"));
    }
}
