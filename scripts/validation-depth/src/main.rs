use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use zip::ZipArchive;

const GITHUB_API_VERSION: &str = "2022-11-28";
const COVERAGE_KIND: &str = "coverage-trend";
const MUTATION_KIND: &str = "mutation-trend";

const CLUSTER_ORDER: &[&str] = &[
    "cow-sdk-core",
    "cow-sdk-contracts",
    "cow-sdk-signing",
    "cow-sdk-app-data",
    "cow-sdk-orderbook",
    "cow-sdk-subgraph",
    "cow-sdk-trading",
    "cow-sdk-browser-wallet",
    "other",
];

const OUTCOME_ORDER: &[&str] = &["CaughtMutant", "MissedMutant", "Unviable", "Timeout"];

#[derive(Debug, Parser)]
#[command(
    about = "Manage retained validation-depth artifacts and summaries for the non-blocking test-depth lane."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Fetch the latest stored trend snapshot artifact when one is available")]
    FetchPreviousArtifact(FetchPreviousArtifactArgs),
    #[command(about = "Build a retained coverage trend snapshot and markdown summary")]
    CoverageTrend(CoverageTrendArgs),
    #[command(about = "Build a retained mutation trend snapshot and markdown summary")]
    MutationTrend(MutationTrendArgs),
}

#[derive(Debug, Args)]
struct FetchPreviousArtifactArgs {
    #[arg(long)]
    repo: String,
    #[arg(long)]
    workflow: String,
    #[arg(long = "artifact-name")]
    artifact_name: String,
    #[arg(long = "output-dir")]
    output_dir: PathBuf,
    #[arg(long, default_value = "")]
    branch: String,
    #[arg(long = "exclude-run-id", default_value = "")]
    exclude_run_id: String,
}

#[derive(Debug, Args)]
struct CoverageTrendArgs {
    #[arg(long)]
    current: PathBuf,
    #[arg(long = "output-md")]
    output_md: PathBuf,
    #[arg(long = "output-json")]
    output_json: PathBuf,
    #[arg(long)]
    previous: Option<PathBuf>,
    #[arg(long = "repo-root", default_value = ".")]
    repo_root: PathBuf,
}

#[derive(Debug, Args)]
struct MutationTrendArgs {
    #[arg(long)]
    scope: String,
    #[arg(long)]
    current: PathBuf,
    #[arg(long = "output-md")]
    output_md: PathBuf,
    #[arg(long = "output-json")]
    output_json: PathBuf,
    #[arg(long)]
    previous: Option<PathBuf>,
    #[arg(long = "exit-code", default_value = "")]
    exit_code: String,
}

#[derive(Debug, Deserialize)]
struct WorkflowRunsResponse {
    #[serde(default)]
    workflow_runs: Vec<WorkflowRun>,
}

#[derive(Debug, Deserialize)]
struct WorkflowRun {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct ArtifactsResponse {
    #[serde(default)]
    artifacts: Vec<Artifact>,
}

#[derive(Debug, Deserialize)]
struct Artifact {
    id: u64,
    name: String,
    #[serde(default)]
    expired: bool,
    archive_download_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CoverageMetric {
    covered: u64,
    count: u64,
    percent: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CoverageBucket {
    covered: u64,
    count: u64,
    uncovered: u64,
    percent: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct CoverageSnapshot {
    kind: String,
    version: u32,
    totals: BTreeMap<String, CoverageMetric>,
    clusters: BTreeMap<String, CoverageBucket>,
    files: BTreeMap<String, CoverageBucket>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MutationBaseline {
    summary: String,
    build_duration: Option<f64>,
    test_duration: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct MutationSnapshot {
    kind: String,
    version: u32,
    scope: String,
    exit_code: String,
    counts: BTreeMap<String, u64>,
    baseline: MutationBaseline,
    survivors: Vec<String>,
    timeouts: Vec<String>,
}

enum FetchOutcome {
    Skip(String),
    Fail(String),
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::FetchPreviousArtifact(args) => fetch_previous_artifact(args),
        Command::CoverageTrend(args) => coverage_trend(args),
        Command::MutationTrend(args) => mutation_trend(args),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn fetch_previous_artifact(args: FetchPreviousArtifactArgs) -> Result<(), String> {
    if args.output_dir.exists() {
        fs::remove_dir_all(&args.output_dir)
            .map_err(|error| format!("failed to clear {}: {error}", args.output_dir.display()))?;
    }

    let Some(token) = std::env::var("GITHUB_TOKEN")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("GH_TOKEN")
                .ok()
                .filter(|value| !value.is_empty())
        })
    else {
        write_fetch_metadata(
            &args.output_dir,
            json!({
                "status": "skipped",
                "reason": "missing GitHub token",
                "artifact_name": args.artifact_name,
            }),
        )?;
        println!("Skipped previous artifact lookup because no GitHub token was available.");
        return Ok(());
    };

    let client = github_client(&token)?;
    let run = match find_latest_run(&client, &args)? {
        Some(run) => run,
        None => {
            write_fetch_metadata(
                &args.output_dir,
                json!({
                    "status": "not_found",
                    "reason": "no prior successful workflow run matched the search",
                    "artifact_name": args.artifact_name,
                }),
            )?;
            println!("No prior successful workflow run matched the search.");
            return Ok(());
        }
    };

    let artifact = match find_matching_artifact(&client, &args.repo, run.id, &args.artifact_name)? {
        Some(artifact) => artifact,
        None => {
            write_fetch_metadata(
                &args.output_dir,
                json!({
                    "status": "not_found",
                    "reason": "matching artifact was not present on the prior run",
                    "artifact_name": args.artifact_name,
                    "run_id": run.id,
                }),
            )?;
            println!("No matching prior artifact was found.");
            return Ok(());
        }
    };

    match download_and_extract_artifact(&client, &artifact.archive_download_url, &args.output_dir) {
        Ok(()) => {
            write_fetch_metadata(
                &args.output_dir,
                json!({
                    "status": "downloaded",
                    "artifact_name": args.artifact_name,
                    "run_id": run.id,
                    "artifact_id": artifact.id,
                }),
            )?;
            println!("Downloaded {} from run {}.", args.artifact_name, run.id);
            Ok(())
        }
        Err(FetchOutcome::Skip(reason)) => {
            write_fetch_metadata(
                &args.output_dir,
                json!({
                    "status": "skipped",
                    "reason": reason,
                    "artifact_name": args.artifact_name,
                }),
            )?;
            println!("Skipped previous artifact lookup.");
            Ok(())
        }
        Err(FetchOutcome::Fail(error)) => Err(error),
    }
}

fn coverage_trend(args: CoverageTrendArgs) -> Result<(), String> {
    let snapshot = build_coverage_snapshot(&args.current, &args.repo_root)?;
    let previous = load_optional_json::<CoverageSnapshot>(args.previous.as_deref())?;
    write_json(&args.output_json, &snapshot)?;
    write_text(
        &args.output_md,
        &coverage_summary(&snapshot, previous.as_ref()),
    )?;
    Ok(())
}

fn mutation_trend(args: MutationTrendArgs) -> Result<(), String> {
    let snapshot = build_mutation_snapshot(&args.scope, &args.current, &args.exit_code)?;
    let previous = load_optional_json::<MutationSnapshot>(args.previous.as_deref())?;
    write_json(&args.output_json, &snapshot)?;
    write_text(
        &args.output_md,
        &mutation_summary(&snapshot, previous.as_ref()),
    )?;
    Ok(())
}

fn github_client(token: &str) -> Result<Client, String> {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static(GITHUB_API_VERSION),
    );
    let auth = HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|error| format!("failed to build authorization header: {error}"))?;
    headers.insert(AUTHORIZATION, auth);
    Client::builder()
        .default_headers(headers)
        .user_agent("cow-rs-validation-depth/1")
        .build()
        .map_err(|error| format!("failed to build GitHub client: {error}"))
}

fn find_latest_run(
    client: &Client,
    args: &FetchPreviousArtifactArgs,
) -> Result<Option<WorkflowRun>, String> {
    let mut url = format!(
        "https://api.github.com/repos/{}/actions/workflows/{}/runs?status=success&exclude_pull_requests=true&per_page=20",
        args.repo,
        urlencoding::encode(&args.workflow),
    );
    if !args.branch.is_empty() {
        url.push_str("&branch=");
        url.push_str(&urlencoding::encode(&args.branch));
    }
    let response = client.get(url).send().map_err(fetch_request_error)?;
    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned HTTP {} while listing workflow runs",
            response.status().as_u16()
        ));
    }
    let payload: WorkflowRunsResponse = response
        .json()
        .map_err(|error| format!("failed to decode workflow-runs response: {error}"))?;
    Ok(payload
        .workflow_runs
        .into_iter()
        .find(|run| args.exclude_run_id.is_empty() || run.id.to_string() != args.exclude_run_id))
}

fn find_matching_artifact(
    client: &Client,
    repo: &str,
    run_id: u64,
    artifact_name: &str,
) -> Result<Option<Artifact>, String> {
    let url =
        format!("https://api.github.com/repos/{repo}/actions/runs/{run_id}/artifacts?per_page=100");
    let response = client.get(url).send().map_err(fetch_request_error)?;
    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned HTTP {} while listing artifacts",
            response.status().as_u16()
        ));
    }
    let payload: ArtifactsResponse = response
        .json()
        .map_err(|error| format!("failed to decode artifact response: {error}"))?;
    Ok(payload
        .artifacts
        .into_iter()
        .find(|artifact| artifact.name == artifact_name && !artifact.expired))
}

fn download_and_extract_artifact(
    client: &Client,
    url: &str,
    output_dir: &Path,
) -> Result<(), FetchOutcome> {
    let response = client.get(url).send().map_err(fetch_request_error_skip)?;
    if !response.status().is_success() {
        return Err(FetchOutcome::Skip(format!(
            "GitHub API returned HTTP {}",
            response.status().as_u16()
        )));
    }

    let bytes = response
        .bytes()
        .map_err(|error| FetchOutcome::Skip(error.to_string()))?;
    fs::create_dir_all(output_dir).map_err(|error| {
        FetchOutcome::Fail(format!(
            "failed to create {}: {error}",
            output_dir.display()
        ))
    })?;

    let cursor = Cursor::new(bytes.to_vec());
    let mut archive = ZipArchive::new(cursor)
        .map_err(|error| FetchOutcome::Fail(format!("failed to read artifact zip: {error}")))?;
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|error| FetchOutcome::Fail(format!("failed to open zip entry: {error}")))?;
        let Some(relative_path) = file.enclosed_name().map(|value| value.to_owned()) else {
            return Err(FetchOutcome::Fail(
                "artifact zip contained an unsafe path".to_owned(),
            ));
        };
        let destination = output_dir.join(relative_path);
        if file.is_dir() {
            fs::create_dir_all(&destination).map_err(|error| {
                FetchOutcome::Fail(format!(
                    "failed to create {}: {error}",
                    destination.display()
                ))
            })?;
            continue;
        }
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                FetchOutcome::Fail(format!("failed to create {}: {error}", parent.display()))
            })?;
        }
        let mut output = fs::File::create(&destination).map_err(|error| {
            FetchOutcome::Fail(format!(
                "failed to create {}: {error}",
                destination.display()
            ))
        })?;
        std::io::copy(&mut file, &mut output).map_err(|error| {
            FetchOutcome::Fail(format!(
                "failed to write {}: {error}",
                destination.display()
            ))
        })?;
        output.flush().map_err(|error| {
            FetchOutcome::Fail(format!(
                "failed to flush {}: {error}",
                destination.display()
            ))
        })?;
    }
    Ok(())
}

fn fetch_request_error(error: reqwest::Error) -> String {
    if let Some(status) = error.status() {
        format!("GitHub API returned HTTP {}", status.as_u16())
    } else {
        format!("GitHub API request failed: {error}")
    }
}

fn fetch_request_error_skip(error: reqwest::Error) -> FetchOutcome {
    if let Some(status) = error.status() {
        FetchOutcome::Skip(format!("GitHub API returned HTTP {}", status.as_u16()))
    } else {
        FetchOutcome::Skip(error.to_string())
    }
}

fn write_fetch_metadata(output_dir: &Path, payload: Value) -> Result<(), String> {
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;
    write_text(
        &output_dir.join("fetch-metadata.json"),
        &format_json(&payload)?,
    )
}

fn build_coverage_snapshot(
    current_path: &Path,
    repo_root: &Path,
) -> Result<CoverageSnapshot, String> {
    let report = read_json_file(current_path)?;
    let data = report
        .get("data")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .ok_or_else(|| "llvm-cov summary did not contain a data entry".to_owned())?;

    let mut totals = BTreeMap::new();
    for metric in ["lines", "functions", "regions"] {
        let section = value_at(data, &["totals", metric])
            .ok_or_else(|| format!("llvm-cov summary missing totals.{metric}"))?;
        totals.insert(
            metric.to_owned(),
            CoverageMetric {
                covered: get_u64(section, "covered")?,
                count: get_u64(section, "count")?,
                percent: get_f64(section, "percent")?,
            },
        );
    }

    let repo_root = repo_root
        .canonicalize()
        .map_err(|error| format!("failed to resolve {}: {error}", repo_root.display()))?;
    let mut clusters: BTreeMap<String, CoverageBucket> = BTreeMap::new();
    let mut files: BTreeMap<String, CoverageBucket> = BTreeMap::new();
    for entry in data
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| "llvm-cov summary missing files".to_owned())?
    {
        let filename = entry
            .get("filename")
            .and_then(Value::as_str)
            .ok_or_else(|| "llvm-cov file entry missing filename".to_owned())?;
        let display_name = display_path(filename, &repo_root);
        let lines = value_at(entry, &["summary", "lines"]).ok_or_else(|| {
            format!("llvm-cov file entry missing summary.lines for {display_name}")
        })?;
        let covered = get_u64(lines, "covered")?;
        let count = get_u64(lines, "count")?;
        let percent = get_f64(lines, "percent")?;
        let uncovered = count.saturating_sub(covered);
        let bucket = CoverageBucket {
            covered,
            count,
            uncovered,
            percent,
        };
        files.insert(display_name.clone(), bucket.clone());
        let cluster = clusters
            .entry(coverage_cluster_for(&display_name).to_owned())
            .or_insert(CoverageBucket {
                covered: 0,
                count: 0,
                uncovered: 0,
                percent: 100.0,
            });
        cluster.covered += covered;
        cluster.count += count;
    }

    for cluster in clusters.values_mut() {
        cluster.uncovered = cluster.count.saturating_sub(cluster.covered);
        cluster.percent = if cluster.count == 0 {
            100.0
        } else {
            cluster.covered as f64 / cluster.count as f64 * 100.0
        };
    }

    Ok(CoverageSnapshot {
        kind: COVERAGE_KIND.to_owned(),
        version: 1,
        totals,
        clusters,
        files,
    })
}

fn coverage_summary(current: &CoverageSnapshot, previous: Option<&CoverageSnapshot>) -> String {
    let mut lines = vec![
        "## Coverage Trend".to_owned(),
        String::new(),
        "| Metric | Current | Delta |".to_owned(),
        "| --- | ---: | ---: |".to_owned(),
    ];

    for metric in ["lines", "functions", "regions"] {
        if let Some(current_metric) = current.totals.get(metric) {
            let delta = previous
                .and_then(|snapshot| snapshot.totals.get(metric))
                .map(|previous_metric| current_metric.percent - previous_metric.percent);
            lines.push(format!(
                "| {} | {:.2}% ({}/{}) | {} |",
                metric,
                current_metric.percent,
                current_metric.covered,
                current_metric.count,
                delta
                    .map(format_f64_delta)
                    .unwrap_or_else(|| "baseline".to_owned())
            ));
        }
    }

    lines.push(String::new());
    lines.push("### Cluster Movement".to_owned());
    lines.push(String::new());
    lines.push("| Cluster | Current | Delta | Uncovered |".to_owned());
    lines.push("| --- | ---: | ---: | ---: |".to_owned());

    let mut seen = BTreeSet::new();
    for cluster in CLUSTER_ORDER {
        if let Some(current_bucket) = current.clusters.get(*cluster) {
            seen.insert((*cluster).to_owned());
            let previous_bucket = previous.and_then(|snapshot| snapshot.clusters.get(*cluster));
            lines.push(format!(
                "| {} | {:.2}% | {} | {} |",
                cluster,
                current_bucket.percent,
                previous_bucket
                    .map(|bucket| format_f64_delta(current_bucket.percent - bucket.percent))
                    .unwrap_or_else(|| "baseline".to_owned()),
                previous_bucket
                    .map(|bucket| format_u64_delta(
                        current_bucket.uncovered as i64 - bucket.uncovered as i64
                    ))
                    .unwrap_or_else(|| current_bucket.uncovered.to_string())
            ));
        }
    }

    for (cluster, current_bucket) in &current.clusters {
        if seen.contains(cluster) {
            continue;
        }
        let previous_bucket = previous.and_then(|snapshot| snapshot.clusters.get(cluster));
        lines.push(format!(
            "| {} | {:.2}% | {} | {} |",
            cluster,
            current_bucket.percent,
            previous_bucket
                .map(|bucket| format_f64_delta(current_bucket.percent - bucket.percent))
                .unwrap_or_else(|| "baseline".to_owned()),
            previous_bucket
                .map(|bucket| format_u64_delta(
                    current_bucket.uncovered as i64 - bucket.uncovered as i64
                ))
                .unwrap_or_else(|| current_bucket.uncovered.to_string())
        ));
    }

    let mut regressions = Vec::new();
    for (path, current_bucket) in &current.files {
        let previous_uncovered = previous
            .and_then(|snapshot| snapshot.files.get(path))
            .map(|bucket| bucket.uncovered)
            .unwrap_or(0);
        if current_bucket.uncovered > previous_uncovered {
            regressions.push((
                current_bucket.uncovered - previous_uncovered,
                current_bucket.uncovered,
                path.clone(),
            ));
        }
    }
    regressions.sort_by(|left, right| right.cmp(left));

    lines.push(String::new());
    lines.push("### New Or Worsened Uncovered Files".to_owned());
    if regressions.is_empty() {
        lines.push(String::new());
        lines.push("- None.".to_owned());
    } else {
        for (delta, uncovered, path) in regressions.into_iter().take(10) {
            lines.push(format!(
                "- `{path}`: +{delta} uncovered lines ({uncovered} total)"
            ));
        }
    }

    if previous.is_none() {
        lines.push(String::new());
        lines.push("_No previous retained coverage snapshot was available._".to_owned());
    }

    lines.push(String::new());
    lines.join("\n")
}

fn build_mutation_snapshot(
    scope: &str,
    current_path: &Path,
    exit_code: &str,
) -> Result<MutationSnapshot, String> {
    let report = read_json_file(current_path)?;
    let outcomes = report
        .get("outcomes")
        .and_then(Value::as_array)
        .ok_or_else(|| "mutation report did not contain outcomes".to_owned())?;

    let mut counts = BTreeMap::new();
    for outcome in OUTCOME_ORDER {
        counts.insert((*outcome).to_owned(), 0);
    }

    let mut baseline = MutationBaseline {
        summary: "missing".to_owned(),
        build_duration: None,
        test_duration: None,
    };
    let mut survivors = Vec::new();
    let mut timeouts = Vec::new();

    for outcome in outcomes {
        let summary = outcome
            .get("summary")
            .and_then(Value::as_str)
            .ok_or_else(|| "mutation outcome missing summary".to_owned())?;

        let scenario = outcome
            .get("scenario")
            .ok_or_else(|| "mutation outcome missing scenario".to_owned())?;
        if scenario.as_str() == Some("Baseline") {
            baseline.summary = summary.to_owned();
            baseline.build_duration = phase_duration(outcome, "Build");
            baseline.test_duration = phase_duration(outcome, "Test");
            continue;
        }

        *counts.entry(summary.to_owned()).or_insert(0) += 1;

        let name = scenario
            .get("Mutant")
            .and_then(|mutant| mutant.get("name"))
            .and_then(Value::as_str)
            .unwrap_or("<unnamed mutant>")
            .to_owned();
        match summary {
            "MissedMutant" => survivors.push(name),
            "Timeout" => timeouts.push(name),
            _ => {}
        }
    }

    survivors.sort();
    timeouts.sort();

    Ok(MutationSnapshot {
        kind: MUTATION_KIND.to_owned(),
        version: 1,
        scope: scope.to_owned(),
        exit_code: exit_code.to_owned(),
        counts,
        baseline,
        survivors,
        timeouts,
    })
}

fn mutation_summary(current: &MutationSnapshot, previous: Option<&MutationSnapshot>) -> String {
    let mut lines = vec![
        format!("## Mutation Trend ({})", current.scope),
        String::new(),
        "| Outcome | Current | Delta |".to_owned(),
        "| --- | ---: | ---: |".to_owned(),
    ];

    for outcome in OUTCOME_ORDER {
        let current_count = *current.counts.get(*outcome).unwrap_or(&0);
        let delta = previous.map(|snapshot| {
            current_count as i64 - *snapshot.counts.get(*outcome).unwrap_or(&0) as i64
        });
        lines.push(format!(
            "| {} | {} | {} |",
            outcome_label(outcome),
            current_count,
            delta
                .map(format_u64_delta)
                .unwrap_or_else(|| "baseline".to_owned())
        ));
    }

    lines.push(String::new());
    lines.push(format!(
        "- baseline: {} (build {}, test {})",
        current.baseline.summary,
        current
            .baseline
            .build_duration
            .map(|value| format!("{value:.1}s"))
            .unwrap_or_else(|| "n/a".to_owned()),
        current
            .baseline
            .test_duration
            .map(|value| format!("{value:.1}s"))
            .unwrap_or_else(|| "n/a".to_owned())
    ));
    if !current.exit_code.is_empty() {
        lines.push(format!(
            "- cargo-mutants exit code: `{}`",
            current.exit_code
        ));
    }

    let previous_survivors: BTreeSet<_> = previous
        .map(|snapshot| snapshot.survivors.iter().cloned().collect())
        .unwrap_or_default();
    let previous_timeouts: BTreeSet<_> = previous
        .map(|snapshot| snapshot.timeouts.iter().cloned().collect())
        .unwrap_or_default();

    let new_survivors: Vec<_> = current
        .survivors
        .iter()
        .filter(|name| !previous_survivors.contains(*name))
        .cloned()
        .collect();
    let new_timeouts: Vec<_> = current
        .timeouts
        .iter()
        .filter(|name| !previous_timeouts.contains(*name))
        .cloned()
        .collect();

    lines.push(String::new());
    lines.push("### New Surviving Mutants".to_owned());
    if new_survivors.is_empty() {
        lines.push("- None.".to_owned());
    } else {
        for survivor in new_survivors.iter().take(10) {
            lines.push(format!("- `{survivor}`"));
        }
    }

    lines.push(String::new());
    lines.push("### New Timeouts".to_owned());
    if new_timeouts.is_empty() {
        lines.push("- None.".to_owned());
    } else {
        for timeout in new_timeouts.iter().take(10) {
            lines.push(format!("- `{timeout}`"));
        }
    }

    if previous.is_none() {
        lines.push(String::new());
        lines.push("_No previous retained mutation snapshot was available._".to_owned());
    }

    lines.push(String::new());
    lines.join("\n")
}

fn load_optional_json<T>(path: Option<&Path>) -> Result<Option<T>, String>
where
    T: DeserializeOwned,
{
    let Some(path) = path else {
        return Ok(None);
    };
    if !path.exists() {
        return Ok(None);
    }
    let value = read_json_file(path)?;
    serde_json::from_value(value)
        .map(Some)
        .map_err(|error| format!("failed to decode {}: {error}", path.display()))
}

fn read_json_file(path: &Path) -> Result<Value, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("failed to decode {} as JSON: {error}", path.display()))
}

fn write_text(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    fs::write(path, contents)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let json = serde_json::to_value(value)
        .map_err(|error| format!("failed to encode {}: {error}", path.display()))?;
    write_text(path, &format_json(&json)?)
}

fn format_json(value: &Value) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|error| format!("failed to format JSON: {error}"))
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

fn get_u64(value: &Value, field: &str) -> Result<u64, String> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing integer field {field}"))
}

fn get_f64(value: &Value, field: &str) -> Result<f64, String> {
    value
        .get(field)
        .and_then(Value::as_f64)
        .ok_or_else(|| format!("missing numeric field {field}"))
}

fn display_path(filename: &str, repo_root: &Path) -> String {
    let normalized_filename = filename.replace('\\', "/");
    let normalized_filename = normalized_filename
        .strip_prefix("//?/")
        .unwrap_or(&normalized_filename)
        .to_owned();

    let root_string = repo_root.to_string_lossy().replace('\\', "/");
    let normalized_root = root_string
        .strip_prefix("//?/")
        .unwrap_or(&root_string)
        .trim_end_matches('/')
        .to_owned();

    let prefix = format!("{normalized_root}/");
    normalized_filename
        .strip_prefix(&prefix)
        .unwrap_or(&normalized_filename)
        .to_owned()
}

fn coverage_cluster_for(path: &str) -> &'static str {
    match path {
        value if value.starts_with("crates/core/") => "cow-sdk-core",
        value if value.starts_with("crates/contracts/") => "cow-sdk-contracts",
        value if value.starts_with("crates/signing/") => "cow-sdk-signing",
        value if value.starts_with("crates/app-data/") => "cow-sdk-app-data",
        value if value.starts_with("crates/orderbook/") => "cow-sdk-orderbook",
        value if value.starts_with("crates/subgraph/") => "cow-sdk-subgraph",
        value if value.starts_with("crates/trading/") => "cow-sdk-trading",
        value if value.starts_with("crates/browser-wallet/") => "cow-sdk-browser-wallet",
        _ => "other",
    }
}

fn phase_duration(outcome: &Value, phase: &str) -> Option<f64> {
    outcome
        .get("phase_results")
        .and_then(Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .find(|item| item.get("phase").and_then(Value::as_str) == Some(phase))
        })
        .and_then(|item| item.get("duration"))
        .and_then(Value::as_f64)
}

fn format_f64_delta(delta: f64) -> String {
    if delta > 0.0 {
        format!("+{delta:.2}")
    } else if delta < 0.0 {
        format!("{delta:.2}")
    } else {
        "0.00".to_owned()
    }
}

fn format_u64_delta(delta: i64) -> String {
    if delta > 0 {
        format!("+{delta}")
    } else {
        delta.to_string()
    }
}

fn outcome_label(value: &str) -> &'static str {
    match value {
        "CaughtMutant" => "Caught",
        "MissedMutant" => "Missed",
        "Unviable" => "Unviable",
        "Timeout" => "Timeout",
        _ => "Other",
    }
}
