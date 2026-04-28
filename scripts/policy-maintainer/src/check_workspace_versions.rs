use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use semver::Version;

use crate::{
    diagnostics::{Diagnostic, OutputMode},
    workspace,
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberVersion {
    pub manifest: String,
    pub package_name: String,
    pub uses_workspace_version: bool,
    pub explicit_version: Option<String>,
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
    let (workspace_version, members) = read_workspace_versions(&args.repo_root)?;
    let errors = validate_versions(&workspace_version, &members);
    if errors.is_empty() {
        Diagnostic::info(
            "PM5000",
            format!(
                "workspace version {workspace_version} is aligned across {} crate(s)",
                members.len()
            ),
        )
        .emit(output_mode, writer)?;
        return Ok(());
    }
    for error in &errors {
        Diagnostic::error("PM5001", error).emit(output_mode, writer)?;
    }
    bail!("workspace version alignment has {} error(s)", errors.len())
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

fn read_workspace_versions(repo_root: &Path) -> anyhow::Result<(String, Vec<MemberVersion>)> {
    let root_manifest = repo_root.join("Cargo.toml");
    let root_text = workspace::read_to_string(&root_manifest)?;
    let workspace_version = find_toml_string(&root_text, "workspace.package", "version")
        .context("root Cargo.toml missing workspace.package.version")?;
    let members =
        workspace_members(&root_text).context("root Cargo.toml missing workspace.members")?;

    let mut output = Vec::new();
    for member in members {
        let manifest = repo_root.join(&member).join("Cargo.toml");
        let text = workspace::read_to_string(&manifest)?;
        let package_name = find_toml_string(&text, "package", "name").unwrap_or(member.clone());
        let uses_workspace_version =
            find_toml_bool(&text, "package", "version.workspace").unwrap_or(false);
        let explicit_version = find_toml_string(&text, "package", "version");
        output.push(MemberVersion {
            manifest: workspace::relative_path(repo_root, &manifest),
            package_name,
            uses_workspace_version,
            explicit_version,
        });
    }
    output.sort_by(|left, right| left.manifest.cmp(&right.manifest));
    Ok((workspace_version, output))
}

fn find_toml_string(text: &str, section: &str, key: &str) -> Option<String> {
    find_toml_value(text, section, key).map(|value| value.trim_matches('"').to_owned())
}

fn find_toml_bool(text: &str, section: &str, key: &str) -> Option<bool> {
    find_toml_value(text, section, key).and_then(|value| match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    })
}

fn find_toml_value<'a>(text: &'a str, section: &str, key: &str) -> Option<&'a str> {
    let mut active = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            active = line == format!("[{section}]");
            continue;
        }
        if !active {
            continue;
        }
        let Some((left, right)) = line.split_once('=') else {
            continue;
        };
        if left.trim() == key {
            return Some(right.trim());
        }
    }
    None
}

fn workspace_members(text: &str) -> Option<Vec<String>> {
    let mut active = false;
    let mut in_members = false;
    let mut members = Vec::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            active = line == "[workspace]";
            continue;
        }
        if !active {
            continue;
        }
        if line.starts_with("members") && line.contains('[') {
            in_members = true;
            continue;
        }
        if in_members {
            if line.starts_with(']') {
                break;
            }
            let value = line.trim_end_matches(',').trim().trim_matches('"');
            if !value.is_empty() {
                members.push(value.to_owned());
            }
        }
    }
    (!members.is_empty()).then_some(members)
}
