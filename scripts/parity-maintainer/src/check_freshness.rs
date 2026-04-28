use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use clap::Args;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::{RepositoryEntry, load_source_lock};

const DEFAULT_GITHUB_API_ROOT: &str = "https://api.github.com";

#[derive(Debug, Args)]
pub(crate) struct CheckFreshnessArgs {
    #[arg(long, default_value = crate::DEFAULT_SOURCE_LOCK)]
    source_lock: PathBuf,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, default_value = DEFAULT_GITHUB_API_ROOT)]
    github_api_root: String,
    #[arg(long, hide = true)]
    now: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct CommitResponse {
    sha: String,
    commit: CommitEnvelope,
}

#[derive(Debug, Deserialize)]
struct CommitEnvelope {
    committer: CommitDate,
}

#[derive(Debug, Deserialize)]
struct CommitDate {
    date: DateTime<Utc>,
}

struct FreshnessRow {
    repo: String,
    pinned: String,
    upstream_head: String,
    upstream_head_date: String,
    lock_age_days: i64,
    status: String,
    note: String,
}

pub(crate) fn run(args: CheckFreshnessArgs) -> Result<()> {
    let lock = load_source_lock(&args.source_lock)?;
    let generated_at = DateTime::parse_from_rfc3339(&lock.meta.generated_at_utc)
        .with_context(|| {
            format!(
                "source-lock generated_at_utc is not RFC3339: {}",
                lock.meta.generated_at_utc
            )
        })?
        .with_timezone(&Utc);
    let now = args.now.unwrap_or_else(Utc::now);
    let lock_age_days = now.signed_duration_since(generated_at).num_days();

    let client = Client::builder()
        .user_agent("cow-rs-parity-maintainer")
        .build()
        .context("failed to build GitHub API client")?;
    let rows = lock
        .repositories
        .iter()
        .map(|repo| freshness_row(&client, &args.github_api_root, repo, lock_age_days))
        .collect::<Vec<_>>();

    let report = render_report(&args.source_lock.display().to_string(), &rows);
    if let Some(output) = args.output {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&output, &report)
            .with_context(|| format!("failed to write {}", output.display()))?;
    }
    print!("{report}");
    Ok(())
}

fn freshness_row(
    client: &Client,
    github_api_root: &str,
    repo: &RepositoryEntry,
    lock_age_days: i64,
) -> FreshnessRow {
    match query_upstream_head(client, github_api_root, repo) {
        Ok(head) => {
            let status = if head.sha == repo.commit {
                "current"
            } else if lock_age_days > 90 {
                "stale"
            } else {
                "drift"
            };
            let note = match status {
                "current" => "source-lock pin matches upstream HEAD",
                "stale" => "source-lock pin differs from upstream HEAD and is older than 90 days",
                "drift" => {
                    "source-lock pin differs from upstream HEAD but is within the 90 day freshness window"
                }
                _ => "unclassified freshness state",
            };
            FreshnessRow {
                repo: repo.id.clone(),
                pinned: short_sha(&repo.commit),
                upstream_head: short_sha(&head.sha),
                upstream_head_date: head.commit.committer.date.to_rfc3339(),
                lock_age_days,
                status: status.to_string(),
                note: note.to_string(),
            }
        }
        Err(error) => FreshnessRow {
            repo: repo.id.clone(),
            pinned: short_sha(&repo.commit),
            upstream_head: "unknown".to_string(),
            upstream_head_date: "unknown".to_string(),
            lock_age_days,
            status: "unknown".to_string(),
            note: format!(
                "GitHub API query failed without failing the informational lane: {error:#}"
            ),
        },
    }
}

fn query_upstream_head(
    client: &Client,
    github_api_root: &str,
    repo: &RepositoryEntry,
) -> Result<CommitResponse> {
    let (owner, name) = github_repo(&repo.remote)?;
    let url = format!(
        "{}/repos/{owner}/{name}/commits/main",
        github_api_root.trim_end_matches('/')
    );
    let mut request = client
        .get(url)
        .header("Accept", "application/vnd.github+json");
    if let Ok(token) = env::var("GITHUB_TOKEN").or_else(|_| env::var("GH_TOKEN")) {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .with_context(|| format!("failed to query GitHub API for {}", repo.id))?;
    if !response.status().is_success() {
        return Err(anyhow!(
            "GitHub API returned {} for {}",
            response.status(),
            repo.id
        ));
    }
    response
        .json()
        .with_context(|| format!("failed to decode GitHub API response for {}", repo.id))
}

fn github_repo(remote: &str) -> Result<(String, String)> {
    let normalized = remote
        .trim()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_string();
    if let Some(rest) = normalized.strip_prefix("https://github.com/") {
        return split_owner_repo(rest);
    }
    if let Some(rest) = normalized.strip_prefix("git@github.com:") {
        return split_owner_repo(rest);
    }
    if let Some(rest) = normalized.strip_prefix("ssh://git@github.com/") {
        return split_owner_repo(rest);
    }
    Err(anyhow!("unsupported GitHub remote format: {remote}"))
}

fn split_owner_repo(rest: &str) -> Result<(String, String)> {
    let mut parts = rest.split('/');
    let owner = parts.next().context("missing GitHub owner")?;
    let repo = parts.next().context("missing GitHub repo")?;
    Ok((owner.to_string(), repo.to_string()))
}

fn render_report(source_lock: &str, rows: &[FreshnessRow]) -> String {
    let mut report = format!(
        "# Source-lock Freshness Report\n\nSource lock: `{source_lock}`\n\n| repo | pinned | upstream HEAD | upstream date | lock age days | status | note |\n| --- | --- | --- | --- | ---: | --- | --- |\n"
    );
    for row in rows {
        report.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            row.repo,
            row.pinned,
            row.upstream_head,
            row.upstream_head_date,
            row.lock_age_days,
            row.status,
            row.note
        ));
    }
    report
}

fn short_sha(sha: &str) -> String {
    sha.chars().take(12).collect()
}
