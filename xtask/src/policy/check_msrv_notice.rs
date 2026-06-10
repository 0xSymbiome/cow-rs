use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, bail};

use crate::policy::workspace;

const REQUIRED_NOTICE_DAYS: u64 = 30;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Enforce the 30-day failure behavior outside release-readiness CI.
    #[arg(long)]
    pub enforce: bool,
    /// Treat the current rust-version as the initial public release floor.
    #[arg(long)]
    pub initial_release: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MsrvNotice {
    pub rust_version: String,
    pub age_days: Option<u64>,
    pub enforce: bool,
    pub initial_release: bool,
}

pub fn run_default() -> anyhow::Result<()> {
    run(&Args {
        repo_root: PathBuf::from("."),
        // Local report-only mode; release runs enforce through CI.
        enforce: false,
        initial_release: false,
    })
}

pub fn run(args: &Args) -> anyhow::Result<()> {
    let rust_version = read_workspace_rust_version(&args.repo_root)?;
    let last_change = latest_rust_version_change(&args.repo_root)?;
    let enforce = args.enforce || release_readiness_workflow();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before unix epoch")?
        .as_secs();
    let age_days = last_change.map(|timestamp| now.saturating_sub(timestamp) / 86_400);
    let notice = MsrvNotice {
        rust_version,
        age_days,
        enforce,
        initial_release: args.initial_release,
    };
    let errors = validate_notice(&notice);

    if errors.is_empty() {
        let message = render_notice_message(&notice);
        if enforce {
            println!("{message}");
        } else {
            println!("warning: {message}");
        }
        return Ok(());
    }

    for error in &errors {
        eprintln!("error: {error}");
    }
    bail!("MSRV notice window has {} error(s)", errors.len())
}

pub fn validate_notice(notice: &MsrvNotice) -> Vec<String> {
    if notice.initial_release {
        return Vec::new();
    }
    let Some(age_days) = notice.age_days else {
        return Vec::new();
    };
    if notice.enforce && age_days < REQUIRED_NOTICE_DAYS {
        vec![format!(
            "rust-version {} changed {age_days} day(s) ago; release readiness requires at least {REQUIRED_NOTICE_DAYS} days",
            notice.rust_version
        )]
    } else {
        Vec::new()
    }
}

fn render_notice_message(notice: &MsrvNotice) -> String {
    if notice.initial_release {
        return format!(
            "rust-version {} establishes the initial public release floor; no MSRV bump notice failure emitted",
            notice.rust_version
        );
    }
    notice.age_days.map_or_else(
        || {
            format!(
                "rust-version {} has no git history entry; no release-readiness failure emitted",
                notice.rust_version
            )
        },
        |age_days| {
            format!(
                "rust-version {} last changed {age_days} day(s) ago{}",
                notice.rust_version,
                if notice.enforce {
                    ""
                } else {
                    "; non-release local mode reports without failing"
                }
            )
        },
    )
}

fn read_workspace_rust_version(repo_root: &Path) -> anyhow::Result<String> {
    let text = workspace::read_to_string(&repo_root.join("Cargo.toml"))?;
    workspace::manifest_string(&text, "workspace.package.rust-version")
        .or_else(|| workspace::manifest_string(&text, "package.rust-version"))
        .context("Cargo.toml does not declare rust-version")
}

fn latest_rust_version_change(repo_root: &Path) -> anyhow::Result<Option<u64>> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args([
            "log",
            "-1",
            "--format=%ct",
            "-G",
            "rust-version",
            "--",
            "Cargo.toml",
        ])
        .output()
        .context("failed to invoke git log for rust-version history")?;
    if !output.status.success() {
        bail!(
            "git log failed while reading rust-version history: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let text = String::from_utf8(output.stdout).context("git log output was not UTF-8")?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed
        .parse::<u64>()
        .map(Some)
        .context("git log returned a non-numeric timestamp")
}

fn release_readiness_workflow() -> bool {
    std::env::var("GITHUB_WORKFLOW")
        .map(|workflow| workflow.to_ascii_lowercase().contains("release-readiness"))
        .unwrap_or(false)
}

