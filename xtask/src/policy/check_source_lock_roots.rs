use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Context;
use serde::Deserialize;

use crate::policy::workspace;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Override source-lock path.
    #[arg(long)]
    pub source_lock: Option<PathBuf>,
    /// Local contracts checkout to compare against the source-lock pin.
    #[arg(long)]
    pub contracts_root: Option<PathBuf>,
    /// Local services checkout to compare against the source-lock pin.
    #[arg(long)]
    pub services_root: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct SourceLock {
    repositories: Vec<RepositoryEntry>,
}

#[derive(Debug, Deserialize)]
struct RepositoryEntry {
    id: String,
    remote: String,
    commit: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservedRoot {
    pub requested_root: PathBuf,
    pub resolved_top_level: Option<PathBuf>,
    pub remote: Option<String>,
    pub commit: Option<String>,
}

pub fn run_default() -> anyhow::Result<()> {
    run(Args {
        repo_root: PathBuf::from("."),
        source_lock: None,
        contracts_root: None,
        services_root: None,
    })
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let lock_path = args
        .source_lock
        .unwrap_or_else(|| args.repo_root.join("parity/source-lock.yaml"));
    let lock: SourceLock = serde_norway::from_str(&workspace::read_to_string(&lock_path)?)
        .with_context(|| format!("failed to parse {}", lock_path.display()))?;
    let expected = lock
        .repositories
        .into_iter()
        .map(|entry| (entry.id.clone(), entry))
        .collect::<BTreeMap<_, _>>();

    let roots = [
        ("contracts", args.contracts_root),
        ("services", args.services_root),
    ];
    let mut emitted = 0usize;

    for (repo_id, root) in roots {
        let Some(root) = root else {
            continue;
        };
        emitted += 1;
        let Some(expected_entry) = expected.get(repo_id) else {
            println!("warning: source-lock has no repository entry for `{repo_id}`");
            continue;
        };
        let observed = inspect_git_root(&root);
        for message in messages_for_observed(repo_id, expected_entry, &observed) {
            println!("{message}");
        }
    }

    if emitted == 0 {
        println!("source-lock local-root check skipped because no upstream roots were supplied");
    }

    Ok(())
}

fn messages_for_observed(
    repo_id: &str,
    expected: &RepositoryEntry,
    observed: &ObservedRoot,
) -> Vec<String> {
    let mut messages = Vec::new();

    match &observed.resolved_top_level {
        Some(top_level) if !same_path(top_level, &observed.requested_root) => {
            messages.push(format!(
                "warning: `{repo_id}` root {} resolves to parent git checkout {}; supply an independent upstream checkout",
                observed.requested_root.display(),
                top_level.display(),
            ));
        }
        None => messages.push(format!(
            "warning: `{repo_id}` root {} is not a readable git checkout",
            observed.requested_root.display(),
        )),
        _ => {}
    }

    match &observed.remote {
        Some(remote) if normalize_remote(remote) != normalize_remote(&expected.remote) => {
            messages.push(format!(
                "warning: `{repo_id}` remote mismatch: observed `{remote}`, expected `{}`",
                expected.remote,
            ));
        }
        None => messages.push(format!(
            "warning: `{repo_id}` checkout has no readable origin remote"
        )),
        _ => {}
    }

    match &observed.commit {
        Some(commit) if commit != &expected.commit => messages.push(format!(
            "warning: `{repo_id}` commit mismatch: observed `{commit}`, expected `{}`",
            expected.commit,
        )),
        None => messages.push(format!(
            "warning: `{repo_id}` checkout has no readable HEAD commit"
        )),
        _ => {}
    }

    if messages.is_empty() {
        messages.push(format!("`{repo_id}` local upstream root matches source-lock"));
    }

    messages
}

fn inspect_git_root(root: &Path) -> ObservedRoot {
    ObservedRoot {
        requested_root: root.to_path_buf(),
        resolved_top_level: git_stdout(root, ["rev-parse", "--show-toplevel"]).map(PathBuf::from),
        remote: git_stdout(root, ["remote", "get-url", "origin"]),
        commit: git_stdout(root, ["rev-parse", "HEAD"]),
    }
}

fn git_stdout<const N: usize>(root: &Path, args: [&str; N]) -> Option<String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn same_path(left: &Path, right: &Path) -> bool {
    canonical_path(left) == canonical_path(right)
}

fn canonical_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn normalize_remote(remote: &str) -> String {
    let trimmed = remote.trim().trim_end_matches('/');
    let https = trimmed.strip_prefix("git@github.com:").map_or_else(
        || trimmed.to_owned(),
        |repo| format!("https://github.com/{repo}"),
    );
    https
        .trim_end_matches(".git")
        .replace('\\', "/")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expected_entry() -> RepositoryEntry {
        RepositoryEntry {
            id: "contracts".to_owned(),
            remote: "https://github.com/cowprotocol/contracts.git".to_owned(),
            commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        }
    }

    #[test]
    fn matching_observation_reports_a_match() {
        let observed = ObservedRoot {
            requested_root: PathBuf::from("contracts"),
            resolved_top_level: Some(PathBuf::from("contracts")),
            remote: Some("git@github.com:cowprotocol/contracts.git".to_owned()),
            commit: Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()),
        };

        let messages = messages_for_observed("contracts", &expected_entry(), &observed);

        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("matches source-lock"));
    }

    #[test]
    fn mismatched_observation_warns_without_failing_closed() {
        let observed = ObservedRoot {
            requested_root: PathBuf::from("vendor/contracts"),
            resolved_top_level: Some(PathBuf::from("vendor")),
            remote: Some("https://github.com/example/contracts.git".to_owned()),
            commit: Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned()),
        };

        let messages = messages_for_observed("contracts", &expected_entry(), &observed);
        let rendered = messages.join("\n");

        assert_eq!(messages.len(), 3);
        assert!(rendered.contains("resolves to parent git checkout"));
        assert!(rendered.contains("remote mismatch"));
        assert!(rendered.contains("commit mismatch"));
    }
}
