use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use semver::Version;
use serde::Deserialize;

use crate::policy::workspace;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberVersion {
    pub manifest: String,
    pub uses_workspace_version: bool,
    pub explicit_version: Option<String>,
}

pub fn run_default() -> anyhow::Result<()> {
    run(&Args {
        repo_root: PathBuf::from("."),
    })
}

pub fn run(args: &Args) -> anyhow::Result<()> {
    let (workspace_version, members) = read_workspace_versions(&args.repo_root)?;
    let mut errors = validate_versions(&workspace_version, &members);
    errors.extend(validate_npm_template(&args.repo_root, &workspace_version)?);
    errors.extend(validate_doc_pins(&args.repo_root, &workspace_version)?);
    if errors.is_empty() {
        println!(
            "workspace version {workspace_version} is aligned across {} crate(s), the npm package template, and every documentation install-pin",
            members.len()
        );
        return Ok(());
    }
    for error in &errors {
        eprintln!("error: {error}");
    }
    bail!("workspace version alignment has {} error(s)", errors.len())
}

/// Confirms every documentation install-pin (README snippets, the crates.io
/// badge, the npm install command, the "is published" prose) matches the
/// workspace version, so a release can never ship stale install instructions.
fn validate_doc_pins(repo_root: &Path, workspace_version: &str) -> anyhow::Result<Vec<String>> {
    Ok(crate::version_surface::scan(repo_root)?
        .into_iter()
        .filter(|pin| pin.version != workspace_version)
        .map(|pin| {
            format!(
                "{}:{} pins version {} but the workspace is {workspace_version}",
                workspace::relative_path(repo_root, &pin.file),
                pin.line,
                pin.version
            )
        })
        .collect())
}

/// Confirms the wasm npm package template version matches the workspace version
/// (the wasm crate's own version is validated through member alignment above).
fn validate_npm_template(repo_root: &Path, workspace_version: &str) -> anyhow::Result<Vec<String>> {
    let path = repo_root.join("crates/wasm/npm/package.template.json");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let template: serde_json::Value = serde_json::from_str(&workspace::read_to_string(&path)?)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(
        match template.get("version").and_then(serde_json::Value::as_str) {
            Some(version) if version == workspace_version => Vec::new(),
            Some(version) => vec![format!(
                "npm package template version {version} does not match workspace version {workspace_version}"
            )],
            None => vec!["npm package template is missing a string version field".to_owned()],
        },
    )
}

pub fn validate_versions(workspace_version: &str, members: &[MemberVersion]) -> Vec<String> {
    let mut errors = Vec::new();
    let Ok(workspace_version) = Version::parse(workspace_version) else {
        return vec![format!(
            "workspace version `{workspace_version}` is not valid SemVer"
        )];
    };

    for member in members {
        if workspace_version.major == 0 {
            if !member.uses_workspace_version {
                errors.push(format!(
                    "{} must use `version.workspace = true` while workspace version is pre-1.0",
                    member.manifest
                ));
            }
            continue;
        }

        if member.uses_workspace_version {
            continue;
        }
        let Some(explicit) = &member.explicit_version else {
            errors.push(format!("{} does not declare a version", member.manifest));
            continue;
        };
        match Version::parse(explicit) {
            Ok(version)
                if version.major == workspace_version.major
                    && version.minor == workspace_version.minor => {}
            Ok(version) => errors.push(format!(
                "{} version {version} is not a patch divergence from workspace version {workspace_version}",
                member.manifest
            )),
            Err(error) => errors.push(format!(
                "{} version `{explicit}` is invalid SemVer: {error}",
                member.manifest
            )),
        }
    }
    errors
}

#[derive(Deserialize)]
struct RootManifest {
    workspace: WorkspaceTable,
}

#[derive(Deserialize)]
struct WorkspaceTable {
    package: WorkspacePackage,
    members: Vec<String>,
}

#[derive(Deserialize)]
struct WorkspacePackage {
    version: String,
}

#[derive(Deserialize)]
struct MemberManifest {
    package: Option<MemberPackage>,
}

#[derive(Deserialize)]
struct MemberPackage {
    version: Option<VersionField>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum VersionField {
    Explicit(String),
    Inherited { workspace: bool },
}

fn read_workspace_versions(repo_root: &Path) -> anyhow::Result<(String, Vec<MemberVersion>)> {
    let root_manifest = repo_root.join("Cargo.toml");
    let root: RootManifest = toml::from_str(&workspace::read_to_string(&root_manifest)?)
        .context("failed to parse root Cargo.toml")?;

    let mut output = Vec::new();
    for member in &root.workspace.members {
        let manifest = repo_root.join(member).join("Cargo.toml");
        let parsed: MemberManifest = toml::from_str(&workspace::read_to_string(&manifest)?)
            .with_context(|| format!("failed to parse {}", manifest.display()))?;
        let (uses_workspace_version, explicit_version) =
            match parsed.package.and_then(|package| package.version) {
                Some(VersionField::Inherited { workspace }) => (workspace, None),
                Some(VersionField::Explicit(version)) => (false, Some(version)),
                None => (false, None),
            };
        output.push(MemberVersion {
            manifest: workspace::relative_path(repo_root, &manifest),
            uses_workspace_version,
            explicit_version,
        });
    }
    output.sort_by(|left, right| left.manifest.cmp(&right.manifest));
    Ok((root.workspace.package.version, output))
}
