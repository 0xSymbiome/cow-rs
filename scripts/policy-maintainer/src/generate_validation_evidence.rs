use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use serde::Deserialize;

use crate::{
    classify_release,
    diagnostics::{Diagnostic, OutputMode},
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root used to resolve default input paths.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Release version to record in the evidence artefact.
    #[arg(long)]
    pub release_version: String,
    /// Output markdown path.
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Byte-compare generated output with the output path on disk.
    #[arg(long)]
    pub check: bool,
    /// Override source-lock input path.
    #[arg(long)]
    pub source_lock: Option<PathBuf>,
    /// Override vendored OpenAPI input path.
    #[arg(long)]
    pub openapi: Option<PathBuf>,
    /// Override WASM runner version input path.
    #[arg(long)]
    pub wasm_versions: Option<PathBuf>,
    /// Override deployment provenance input path.
    #[arg(long)]
    pub deployment_provenance: Option<PathBuf>,
    /// Override release-readiness lane status input path.
    #[arg(long)]
    pub lane_status: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct SourceLock {
    meta: SourceLockMeta,
    repositories: Vec<SourceRepository>,
}

#[derive(Debug, Deserialize)]
struct SourceLockMeta {
    generated_at_utc: String,
}

#[derive(Debug, Deserialize)]
struct SourceRepository {
    id: String,
    remote: String,
    commit: String,
    role: String,
}

#[derive(Debug, Deserialize)]
struct WasmVersions {
    chrome_for_testing: ChromeForTesting,
}

#[derive(Debug, Deserialize)]
struct ChromeForTesting {
    channel: String,
    version: String,
    revision: String,
    released_at: String,
}

#[derive(Debug, Deserialize)]
struct DeploymentProvenance {
    generated_at_utc: String,
    provenance: Vec<DeploymentRow>,
}

#[derive(Debug, Deserialize)]
struct DeploymentRow {
    contract_id: String,
    chain_id: u64,
    env: String,
    address: String,
    live_confirmation: Option<LiveConfirmation>,
}

#[derive(Debug, Deserialize)]
struct LiveConfirmation {
    kind: String,
    code_hash: String,
    confirmed_at: String,
}

#[derive(Debug, Deserialize)]
struct LaneStatusInput {
    generated_at_utc: String,
    workflow: WorkflowEvidence,
    lanes: Vec<LaneEvidence>,
}

#[derive(Debug, Deserialize)]
struct WorkflowEvidence {
    name: String,
    file: Option<String>,
    run_url: Option<String>,
    commit_sha: String,
}

#[derive(Clone, Debug, Deserialize)]
struct LaneEvidence {
    lane: String,
    status: String,
    notes: String,
    step_id: String,
}

#[derive(Clone, Debug)]
struct OpenApiEvidence {
    source: String,
    commit: String,
    path: String,
    generated_at: String,
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
    let output = output_path(
        &args.repo_root,
        args.output.as_deref(),
        &args.release_version,
    );
    let document = generate_markdown(&args)?;

    if args.check {
        let current = read_required("output", &output)?;
        if current != document {
            bail!(
                "validation evidence differs from committed output {}",
                output.display()
            );
        }
        Diagnostic::info(
            "PM12000",
            format!("validation evidence matches {}", output.display()),
        )
        .emit(output_mode, writer)?;
        return Ok(());
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&output, document)
        .with_context(|| format!("failed to write {}", output.display()))?;
    Diagnostic::info(
        "PM12001",
        format!("wrote validation evidence to {}", output.display()),
    )
    .emit(output_mode, writer)?;
    Ok(())
}

pub fn generate_markdown(args: &Args) -> anyhow::Result<String> {
    let repo_root = &args.repo_root;
    let source_lock_path = input_path(
        repo_root,
        args.source_lock.as_deref(),
        "parity/source-lock.yaml",
    );
    let openapi_path = input_path(
        repo_root,
        args.openapi.as_deref(),
        "parity/openapi/services-orderbook.yml",
    );
    let wasm_versions_path = input_path(
        repo_root,
        args.wasm_versions.as_deref(),
        ".github/config/wasm-test-versions.yaml",
    );
    let deployment_path = input_path(
        repo_root,
        args.deployment_provenance.as_deref(),
        "crates/contracts/deployment-provenance.yaml",
    );
    let lane_status_path = input_path(
        repo_root,
        args.lane_status.as_deref(),
        &format!(
            ".github/release-evidence/release-readiness-status-{}.yaml",
            args.release_version
        ),
    );

    let source_lock = parse_yaml::<SourceLock>("source-lock", &source_lock_path)?;
    let openapi = parse_openapi_header(&read_required("OpenAPI vendoring", &openapi_path)?)?;
    let wasm_versions = parse_yaml::<WasmVersions>("wasm-test-versions", &wasm_versions_path)?;
    let deployment = parse_yaml::<DeploymentProvenance>("deployment provenance", &deployment_path)?;
    let lane_status = parse_yaml::<LaneStatusInput>("lane status", &lane_status_path)?;
    let classification = classify_release::classify_versions(None, &args.release_version)?;

    let mut repositories = source_lock.repositories;
    repositories.sort_by(|left, right| left.id.cmp(&right.id));

    let mut lanes = lane_status.lanes;
    lanes.sort_by(|left, right| left.lane.cmp(&right.lane));

    let mut deployment_rows = deployment
        .provenance
        .into_iter()
        .filter(|row| row.live_confirmation.is_some())
        .collect::<Vec<_>>();
    deployment_rows.sort_by(|left, right| {
        left.chain_id
            .cmp(&right.chain_id)
            .then_with(|| left.contract_id.cmp(&right.contract_id))
            .then_with(|| left.env.cmp(&right.env))
            .then_with(|| left.address.cmp(&right.address))
    });

    let mut markdown = String::new();
    push_line(
        &mut markdown,
        &format!("# Validation Evidence - cow-rs {}", args.release_version),
    );
    push_line(&mut markdown, "");
    push_line(
        &mut markdown,
        &format!("Generated: {}", lane_status.generated_at_utc),
    );
    push_line(
        &mut markdown,
        &format!("Workflow: {}", lane_status.workflow.name),
    );
    if let Some(file) = optional_text(lane_status.workflow.file.as_deref()) {
        push_line(&mut markdown, &format!("Workflow file: {file}"));
    }
    let workflow_run =
        optional_text(lane_status.workflow.run_url.as_deref()).unwrap_or("pending final run");
    push_line(
        &mut markdown,
        &format!("Workflow run: {workflow_run}"),
    );
    push_line(
        &mut markdown,
        &format!("Candidate commit: {}", lane_status.workflow.commit_sha),
    );
    push_line(
        &mut markdown,
        &format!(
            "Release classification: {} (semver-checks: {})",
            release_kind_name(classification.release_kind),
            semver_mode_name(classification.semver_checks_mode)
        ),
    );
    push_line(&mut markdown, "");

    push_line(&mut markdown, "## Lane Status");
    push_line(&mut markdown, "");
    push_line(&mut markdown, "| Lane | Status | Step | Notes |");
    push_line(&mut markdown, "| --- | --- | --- | --- |");
    for lane in lanes {
        push_line(
            &mut markdown,
            &format!(
                "| {} | {} | {} | {} |",
                lane.lane, lane.status, lane.step_id, lane.notes
            ),
        );
    }
    push_line(&mut markdown, "");

    push_line(&mut markdown, "## Source-Lock");
    push_line(&mut markdown, "");
    push_line(
        &mut markdown,
        &format!("Generated at: {}", source_lock.meta.generated_at_utc),
    );
    push_line(&mut markdown, "");
    push_line(
        &mut markdown,
        "| Repository | Remote | Pinned commit | Role |",
    );
    push_line(&mut markdown, "| --- | --- | --- | --- |");
    for repository in repositories {
        push_line(
            &mut markdown,
            &format!(
                "| {} | {} | {} | {} |",
                repository.id, repository.remote, repository.commit, repository.role
            ),
        );
    }
    push_line(&mut markdown, "");

    push_line(&mut markdown, "## OpenAPI Vendoring");
    push_line(&mut markdown, "");
    push_line(
        &mut markdown,
        "| Source | Path | Pinned commit | Generated at |",
    );
    push_line(&mut markdown, "| --- | --- | --- | --- |");
    push_line(
        &mut markdown,
        &format!(
            "| {} | {} | {} | {} |",
            openapi.source, openapi.path, openapi.commit, openapi.generated_at
        ),
    );
    push_line(&mut markdown, "");

    push_line(&mut markdown, "## WASM Runner");
    push_line(&mut markdown, "");
    push_line(&mut markdown, "| Field | Value |");
    push_line(&mut markdown, "| --- | --- |");
    push_line(
        &mut markdown,
        &format!("| Channel | {} |", wasm_versions.chrome_for_testing.channel),
    );
    push_line(
        &mut markdown,
        &format!(
            "| Chrome version | {} |",
            wasm_versions.chrome_for_testing.version
        ),
    );
    push_line(
        &mut markdown,
        &format!(
            "| ChromeDriver version | {} |",
            wasm_versions.chrome_for_testing.version
        ),
    );
    push_line(
        &mut markdown,
        &format!(
            "| Revision | {} |",
            wasm_versions.chrome_for_testing.revision
        ),
    );
    push_line(
        &mut markdown,
        &format!(
            "| Released at | {} |",
            wasm_versions.chrome_for_testing.released_at
        ),
    );
    push_line(&mut markdown, "");

    push_line(&mut markdown, "## Deployment Provenance");
    push_line(&mut markdown, "");
    push_line(
        &mut markdown,
        &format!("Generated at: {}", deployment.generated_at_utc),
    );
    push_line(&mut markdown, "");
    push_line(
        &mut markdown,
        "| Chain ID | Environment | Contract | Address | Code hash | Confirmed at |",
    );
    push_line(&mut markdown, "| --- | --- | --- | --- | --- | --- |");
    for row in deployment_rows {
        let confirmation = row
            .live_confirmation
            .expect("deployment rows were filtered for confirmations");
        push_line(
            &mut markdown,
            &format!(
                "| {} | {} | {} | {} | {} | {} |",
                row.chain_id,
                row.env,
                row.contract_id,
                row.address,
                confirmation_hash(&confirmation),
                confirmation.confirmed_at
            ),
        );
    }

    Ok(markdown)
}

fn confirmation_hash(confirmation: &LiveConfirmation) -> &str {
    if confirmation.kind == "code_hash" {
        &confirmation.code_hash
    } else {
        ""
    }
}

fn optional_text(value: Option<&str>) -> Option<&str> {
    value.and_then(|text| {
        let trimmed = text.trim();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn input_path(repo_root: &Path, override_path: Option<&Path>, default_path: &str) -> PathBuf {
    match override_path {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(default_path),
    }
}

fn output_path(repo_root: &Path, output: Option<&Path>, release_version: &str) -> PathBuf {
    input_path(
        repo_root,
        output,
        &format!(".github/release-evidence/validation-evidence-{release_version}.md"),
    )
}

fn parse_yaml<T: for<'de> Deserialize<'de>>(label: &str, path: &Path) -> anyhow::Result<T> {
    let content = read_required(label, path)?;
    serde_norway::from_str(&content)
        .with_context(|| format!("failed to parse required input {label}: {}", path.display()))
}

fn read_required(label: &str, path: &Path) -> anyhow::Result<String> {
    if !path.is_file() {
        bail!("missing required input {label}: {}", path.display());
    }
    fs::read_to_string(path)
        .with_context(|| format!("failed to read required input {label}: {}", path.display()))
}

fn parse_openapi_header(text: &str) -> anyhow::Result<OpenApiEvidence> {
    let mut source = None;
    let mut commit = None;
    let mut path = None;
    let mut generated_at = None;

    for line in text.lines().take(8) {
        if let Some(value) = line.strip_prefix("# Vendored from ") {
            let Some((repo, sha)) = value.split_once(" @ ") else {
                bail!("OpenAPI vendoring header is missing ` @ ` separator");
            };
            source = Some(repo.to_owned());
            commit = Some(sha.to_owned());
        } else if let Some(value) = line.strip_prefix("# Path: ") {
            path = Some(value.to_owned());
        } else if let Some(value) = line.strip_prefix("# Generated: ") {
            generated_at = Some(value.to_owned());
        }
    }

    Ok(OpenApiEvidence {
        source: source.context("OpenAPI vendoring header missing source repository")?,
        commit: commit.context("OpenAPI vendoring header missing source commit")?,
        path: path.context("OpenAPI vendoring header missing source path")?,
        generated_at: generated_at
            .context("OpenAPI vendoring header missing generated timestamp")?,
    })
}

fn release_kind_name(kind: classify_release::ReleaseKind) -> &'static str {
    match kind {
        classify_release::ReleaseKind::FirstFunctional => "first_functional",
        classify_release::ReleaseKind::Patch => "patch",
        classify_release::ReleaseKind::Pre1_0Minor => "pre_1_0_minor",
        classify_release::ReleaseKind::Post1_0Minor => "post_1_0_minor",
        classify_release::ReleaseKind::Major => "major",
        classify_release::ReleaseKind::Unsupported => "unsupported",
    }
}

fn semver_mode_name(mode: classify_release::SemverChecksMode) -> &'static str {
    match mode {
        classify_release::SemverChecksMode::Skip => "skip",
        classify_release::SemverChecksMode::Advisory => "advisory",
        classify_release::SemverChecksMode::Blocking => "blocking",
    }
}

fn push_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}
