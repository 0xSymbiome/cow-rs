use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, bail};

use crate::{
    diagnostics::{Diagnostic, DiagnosticLevel, OutputMode},
    workspace,
};

const REQUIRED_NOTICE_DAYS: u64 = 30;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Enforce the 30-day failure behavior outside release-readiness CI.
    #[arg(long)]
    pub enforce: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MsrvNotice {
    pub rust_version: String,
    pub age_days: Option<u64>,
    pub enforce: bool,
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
    };
    let errors = validate_notice(&notice);

    if errors.is_empty() {
        let level = if enforce {
            DiagnosticLevel::Info
        } else {
            DiagnosticLevel::Warn
        };
        Diagnostic::new(level, "PM6000", render_notice_message(&notice))
            .emit(output_mode, writer)?;
        return Ok(());
    }

    for error in &errors {
        Diagnostic::error("PM6001", error).emit(output_mode, writer)?;
    }
    bail!("MSRV notice window has {} error(s)", errors.len())
}

pub fn validate_notice(notice: &MsrvNotice) -> Vec<String> {
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
    match notice.age_days {
        Some(age_days) => format!(
            "rust-version {} last changed {age_days} day(s) ago{}",
            notice.rust_version,
            if notice.enforce {
                ""
            } else {
                "; non-release local mode reports without failing"
            }
        ),
        None => format!(
            "rust-version {} has no git history entry; no release-readiness failure emitted",
            notice.rust_version
        ),
    }
}

fn read_workspace_rust_version(repo_root: &Path) -> anyhow::Result<String> {
    let text = workspace::read_to_string(&repo_root.join("Cargo.toml"))?;
    find_toml_string(&text, "workspace.package", "rust-version")
        .or_else(|| find_toml_string(&text, "package", "rust-version"))
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

#[allow(dead_code)]
fn days_ago(days: u64) -> SystemTime {
    SystemTime::now() - Duration::from_secs(days * 86_400)
}

fn find_toml_string(text: &str, section: &str, key: &str) -> Option<String> {
    let mut active = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            active = line == format!("[{section}]");
            continue;
        }
        if active {
            let Some((left, right)) = line.split_once('=') else {
                continue;
            };
            if left.trim() == key {
                return Some(right.trim().trim_matches('"').to_owned());
            }
        }
    }
    None
}
