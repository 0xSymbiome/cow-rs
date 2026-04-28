use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, bail};
use semver::Version;
use serde::Serialize;

use crate::diagnostics::{Diagnostic, OutputMode};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root used for Cargo.toml and git commands.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Base ref or explicit SemVer string.
    #[arg(long)]
    pub base_ref: Option<String>,
    /// Head ref or explicit SemVer string.
    #[arg(long, default_value = "HEAD")]
    pub head_ref: String,
    /// Workspace Cargo.toml to read for the head side.
    #[arg(long)]
    pub workspace_cargo_toml: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseKind {
    FirstFunctional,
    Patch,
    #[serde(rename = "pre_1_0_minor")]
    Pre1_0Minor,
    #[serde(rename = "post_1_0_minor")]
    Post1_0Minor,
    Major,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SemverChecksMode {
    Skip,
    Advisory,
    Blocking,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Classification {
    pub release_kind: ReleaseKind,
    pub is_first_functional_release: bool,
    pub is_patch: bool,
    pub is_minor: bool,
    pub is_major: bool,
    pub head_version: String,
    pub base_version: String,
    pub baseline_tag: Option<String>,
    pub semver_checks_mode: SemverChecksMode,
}

impl Classification {
    pub fn is_unsupported(&self) -> bool {
        self.release_kind == ReleaseKind::Unsupported
    }
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
    let repo_root = args.repo_root;
    let classification = classify_refs(
        &repo_root,
        args.base_ref.as_deref(),
        &args.head_ref,
        args.workspace_cargo_toml.as_deref(),
    )?;

    if let Some(tag) = &classification.baseline_tag
        && matches!(
            classification.release_kind,
            ReleaseKind::Patch | ReleaseKind::Post1_0Minor
        )
        && !git_tag_exists(&repo_root, tag)?
    {
        bail!("required semver baseline tag `{tag}` does not exist locally");
    }

    match output_mode {
        OutputMode::Json => {
            writeln!(writer, "{}", serde_json::to_string(&classification)?)?;
        }
        OutputMode::Text => {
            let message = format!(
                "release_kind={:?} head={} base={} semver_checks_mode={:?}",
                classification.release_kind,
                classification.head_version,
                classification.base_version,
                classification.semver_checks_mode
            );
            Diagnostic::info("PM1000", message).emit(output_mode, writer)?;
        }
    }
    Ok(())
}

pub fn classify_refs(
    repo_root: &Path,
    base_ref: Option<&str>,
    head_ref: &str,
    workspace_cargo_toml: Option<&Path>,
) -> anyhow::Result<Classification> {
    let head_version = resolve_head_version(repo_root, head_ref, workspace_cargo_toml)?;
    let base_version = match base_ref {
        Some(base_ref) => resolve_base_version(repo_root, base_ref)?,
        None => None,
    };
    classify_versions(base_version.as_deref(), &head_version)
}

pub fn classify_versions(base: Option<&str>, head: &str) -> anyhow::Result<Classification> {
    let head_version =
        Version::parse(head).with_context(|| format!("invalid head version {head}"))?;
    let base_version = base
        .map(|version| {
            Version::parse(version).with_context(|| format!("invalid base version {version}"))
        })
        .transpose()?;

    let (kind, baseline_tag, mode) = match &base_version {
        None if is_first_functional_head(&head_version) => {
            (ReleaseKind::FirstFunctional, None, SemverChecksMode::Skip)
        }
        None => (ReleaseKind::Unsupported, None, SemverChecksMode::Skip),
        Some(base) if is_first_functional_head(&head_version) && is_reserved_placeholder(base) => {
            (ReleaseKind::FirstFunctional, None, SemverChecksMode::Skip)
        }
        Some(base) if is_patch(base, &head_version) => (
            ReleaseKind::Patch,
            Some(format!("v{}.{}.{}", base.major, base.minor, base.patch)),
            SemverChecksMode::Blocking,
        ),
        Some(base) if is_pre_1_0_minor(base, &head_version) => (
            ReleaseKind::Pre1_0Minor,
            Some(format!("v{}.{}.{}", base.major, base.minor, base.patch)),
            SemverChecksMode::Advisory,
        ),
        Some(base) if is_post_1_0_minor(base, &head_version) => (
            ReleaseKind::Post1_0Minor,
            Some(format!("v{}.{}.{}", base.major, base.minor, base.patch)),
            SemverChecksMode::Blocking,
        ),
        Some(base) if is_major(base, &head_version) => {
            (ReleaseKind::Major, None, SemverChecksMode::Skip)
        }
        Some(_) => (ReleaseKind::Unsupported, None, SemverChecksMode::Skip),
    };

    Ok(Classification {
        release_kind: kind,
        is_first_functional_release: kind == ReleaseKind::FirstFunctional,
        is_patch: kind == ReleaseKind::Patch,
        is_minor: matches!(kind, ReleaseKind::Pre1_0Minor | ReleaseKind::Post1_0Minor),
        is_major: kind == ReleaseKind::Major,
        head_version: head_version.to_string(),
        base_version: base_version
            .map_or_else(|| "absent".to_owned(), |version| version.to_string()),
        baseline_tag,
        semver_checks_mode: mode,
    })
}

pub fn workspace_version_from_toml(toml_text: &str) -> anyhow::Result<String> {
    find_toml_string(toml_text, "workspace.package", "version")
        .or_else(|| find_toml_string(toml_text, "package", "version"))
        .context("Cargo.toml does not declare a package or workspace package version")
}

fn resolve_head_version(
    repo_root: &Path,
    head_ref: &str,
    workspace_cargo_toml: Option<&Path>,
) -> anyhow::Result<String> {
    if Version::parse(head_ref).is_ok() {
        return Ok(head_ref.to_owned());
    }
    if head_ref == "HEAD" {
        let path = workspace_cargo_toml
            .map(Path::to_path_buf)
            .unwrap_or_else(|| repo_root.join("Cargo.toml"));
        return workspace_version_from_toml(
            &std::fs::read_to_string(&path).with_context(|| {
                format!("failed to read head Cargo.toml from {}", path.display())
            })?,
        );
    }
    let content = git_show(repo_root, head_ref, "Cargo.toml")?;
    workspace_version_from_toml(&content)
}

fn resolve_base_version(repo_root: &Path, base_ref: &str) -> anyhow::Result<Option<String>> {
    if Version::parse(base_ref).is_ok() {
        return Ok(Some(base_ref.to_owned()));
    }
    match git_show(repo_root, base_ref, "Cargo.toml") {
        Ok(content) => workspace_version_from_toml(&content).map(Some),
        Err(_) => Ok(None),
    }
}

fn git_show(repo_root: &Path, git_ref: &str, path: &str) -> anyhow::Result<String> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["show", &format!("{git_ref}:{path}")])
        .output()
        .context("failed to invoke git show")?;
    if !output.status.success() {
        bail!(
            "git show failed for {git_ref}:{path}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8(output.stdout).context("git show output was not UTF-8")
}

fn git_tag_exists(repo_root: &Path, tag: &str) -> anyhow::Result<bool> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["rev-parse", "--verify", tag])
        .output()
        .context("failed to invoke git rev-parse")?;
    Ok(output.status.success())
}

fn is_first_functional_head(version: &Version) -> bool {
    version.major == 0 && version.minor == 1 && version.patch == 0 && version.pre.is_empty()
}

fn is_reserved_placeholder(version: &Version) -> bool {
    version.major == 0
        && version.minor == 0
        && version.patch <= 1
        && (!version.pre.is_empty() || version.patch == 0)
}

fn is_patch(base: &Version, head: &Version) -> bool {
    head.major == base.major
        && head.minor == base.minor
        && head.patch == base.patch + 1
        && head.pre.is_empty()
}

fn is_pre_1_0_minor(base: &Version, head: &Version) -> bool {
    base.major == 0
        && head.major == 0
        && head.minor == base.minor + 1
        && head.patch == 0
        && head.pre.is_empty()
}

fn is_post_1_0_minor(base: &Version, head: &Version) -> bool {
    base.major >= 1
        && head.major == base.major
        && head.minor == base.minor + 1
        && head.patch == 0
        && head.pre.is_empty()
}

fn is_major(base: &Version, head: &Version) -> bool {
    head.major == base.major + 1 && head.minor == 0 && head.patch == 0 && head.pre.is_empty()
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
