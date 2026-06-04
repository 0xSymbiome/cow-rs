use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

mod openapi_coverage;
mod vendor_openapi;
mod verify_sol_provenance;

const GENERATED_AT_UTC: &str = "2026-04-29T00:00:00Z";
const DEFAULT_SOURCE_LOCK: &str = "parity/source-lock.yaml";

#[derive(Clone, Copy)]
struct RepoTemplate {
    id: &'static str,
    remote: &'static str,
    role: &'static str,
    local_hint: &'static str,
    producer_paths: &'static [&'static str],
}

const CONTRACTS_PATHS: &[&str] = &[
    "src/ts/order.ts",
    "src/ts/sign.ts",
    "src/ts/settlement.ts",
    "src/ts/swap.ts",
    "src/ts/interaction.ts",
    "src/ts/vault.ts",
    "src/ts/proxy.ts",
    "test/GPv2Order/PackOrderUidParams.t.sol",
    "test/GPv2Order/ExtractOrderUidParams.t.sol",
    "test/GPv2Trade/ExtractFlags.t.sol",
    "test/GPv2Trade/ExtractOrder.t.sol",
    "test/GPv2Settlement/OrderRefunds.t.sol",
    "test/GPv2Settlement/Swap/Swap.t.sol",
];

const SERVICES_PATHS: &[&str] = &[
    "crates/orderbook/openapi.yml",
    "crates/shared/src/order_validation.rs",
    "crates/orderbook/src/app_data.rs",
    "crates/model/src/order.rs",
    "crates/orderbook/src/quoter.rs",
];

const ALLOY_PATHS: &[&str] = &[
    "Cargo.toml",
    "crates/consensus/src/lib.rs",
    "crates/json-rpc/src/lib.rs",
    "crates/network/src/lib.rs",
    "crates/provider/src/lib.rs",
    "crates/rpc-types-eth/src/lib.rs",
    "crates/signer/src/lib.rs",
    "crates/signer-local/src/lib.rs",
    "crates/transport/src/lib.rs",
    "crates/transport-http/src/lib.rs",
];

const ALLOY_CORE_PATHS: &[&str] = &[
    "Cargo.toml",
    "crates/dyn-abi/src/lib.rs",
    "crates/json-abi/src/lib.rs",
    "crates/primitives/src/lib.rs",
    "crates/sol-macro/src/lib.rs",
    "crates/sol-types/src/lib.rs",
];

const COMPOSABLE_COW_PATHS: &[&str] = &[
    "networks.json",
    "src/ComposableCoW.sol",
    "src/BaseConditionalOrder.sol",
    "src/types/twap/TWAP.sol",
    "src/types/GoodAfterTime.sol",
    "src/types/StopLoss.sol",
    "src/types/TradeAboveThreshold.sol",
    "src/types/PerpetualStableSwap.sol",
    "src/interfaces/IConditionalOrder.sol",
    "src/interfaces/ISwapGuard.sol",
    "src/interfaces/IValueFactory.sol",
];
const COMPOSABLE_COW_LIB_SAFE_PATHS: &[&str] = &["contracts/handler/ExtensibleFallbackHandler.sol"];
const COW_SHED_PATHS: &[&str] = &[
    "networks.json",
    "src/COWShed.sol",
    "src/COWShedFactory.sol",
    "src/COWShedForComposableCoW.sol",
    "src/COWShedProxy.sol",
    "src/COWShedStorage.sol",
    "src/ERC1271Forwarder.sol",
    "src/interfaces/ICOWAuthHook.sol",
    "src/interfaces/IERC1271.sol",
    "src/LibAuthenticatedHooks.sol",
];
const WATCH_TOWER_PATHS: &[&str] = &[
    "README.md",
    "src/utils/orderBookApi.ts",
    "src/types/index.ts",
];
const ETHFLOWCONTRACT_PATHS: &[&str] = &[
    "src/CoWSwapEthFlow.sol",
    "src/libraries/EthFlowOrder.sol",
    "src/interfaces/ICoWSwapOnchainOrders.sol",
    "src/mixins/CoWSwapOnchainOrders.sol",
    "src/interfaces/IWrappedNativeToken.sol",
];
const HELPER_REPO_TEMPLATES: &[RepoTemplate] = &[
    RepoTemplate {
        id: "contracts",
        remote: "https://github.com/cowprotocol/contracts.git",
        role: "primary",
        local_hint: "<contracts-checkout>",
        producer_paths: CONTRACTS_PATHS,
    },
    RepoTemplate {
        id: "services",
        remote: "https://github.com/cowprotocol/services.git",
        role: "wire-authority",
        local_hint: "<services-checkout>",
        producer_paths: SERVICES_PATHS,
    },
    RepoTemplate {
        id: "composable-cow",
        remote: "https://github.com/cowprotocol/composable-cow.git",
        role: "primary",
        local_hint: "<composable-cow-checkout>",
        producer_paths: COMPOSABLE_COW_PATHS,
    },
    RepoTemplate {
        id: "composable-cow/lib/safe",
        remote: "https://github.com/cowdao-grants/extensible-fallback-handler.git",
        role: "primary-via-submodule",
        local_hint: "<composable-cow-checkout>/lib/safe",
        producer_paths: COMPOSABLE_COW_LIB_SAFE_PATHS,
    },
    RepoTemplate {
        id: "cow-shed",
        remote: "https://github.com/cowdao-grants/cow-shed.git",
        role: "primary",
        local_hint: "<cow-shed-checkout>",
        producer_paths: COW_SHED_PATHS,
    },
    RepoTemplate {
        id: "ethflowcontract",
        remote: "https://github.com/cowprotocol/ethflowcontract.git",
        role: "primary",
        local_hint: "<ethflowcontract-checkout>",
        producer_paths: ETHFLOWCONTRACT_PATHS,
    },
    RepoTemplate {
        id: "watch-tower",
        remote: "https://github.com/cowprotocol/watch-tower.git",
        role: "reference-only",
        local_hint: "<watch-tower-checkout>",
        producer_paths: WATCH_TOWER_PATHS,
    },
];

const REPO_TEMPLATES: &[RepoTemplate] = &[
    RepoTemplate {
        id: "contracts",
        remote: "https://github.com/cowprotocol/contracts.git",
        role: "primary",
        local_hint: "<contracts-checkout>",
        producer_paths: CONTRACTS_PATHS,
    },
    RepoTemplate {
        id: "services",
        remote: "https://github.com/cowprotocol/services.git",
        role: "wire-authority",
        local_hint: "<services-checkout>",
        producer_paths: SERVICES_PATHS,
    },
];

const DEPENDENCY_REPO_TEMPLATES: &[RepoTemplate] = &[
    RepoTemplate {
        id: "alloy",
        remote: "https://github.com/alloy-rs/alloy.git",
        role: "dependency",
        local_hint: "<alloy-checkout>",
        producer_paths: ALLOY_PATHS,
    },
    RepoTemplate {
        id: "alloy-core",
        remote: "https://github.com/alloy-rs/core.git",
        role: "dependency",
        local_hint: "<alloy-core-checkout>",
        producer_paths: ALLOY_CORE_PATHS,
    },
];

#[derive(Debug, Serialize, Deserialize)]
struct SourceLock {
    meta: LockMeta,
    repositories: Vec<RepositoryEntry>,
    fixtures: Vec<FixtureEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LockMeta {
    schema_version: u32,
    generated_at_utc: String,
    purpose: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RepositoryEntry {
    id: String,
    remote: String,
    commit: String,
    role: String,
    optional_local_path: String,
    producer_paths: Vec<String>,
    #[serde(default)]
    pinned_at: Option<String>,
    #[serde(default)]
    pinned_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FixtureEntry {
    surface: String,
    file: String,
    source_refs: Vec<FixtureSourceRef>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FixtureSourceRef {
    repo: String,
    path: String,
}

struct CliOptions {
    source_lock: PathBuf,
    output: PathBuf,
    output_root: Option<PathBuf>,
    contracts_root: Option<PathBuf>,
    services_root: Option<PathBuf>,
}

#[derive(Debug, Parser)]
#[command(name = "parity-maintainer")]
#[command(about = "Maintains source-lock, parity provenance, and OpenAPI coverage artifacts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Snapshot pinned upstream commits into parity/source-lock.yaml.
    Snapshot(UpstreamRootsArgs),
    /// Validate source-lock and committed parity fixture provenance.
    Validate(ValidateArgs),
    /// Provision source-lock-pinned upstream checkouts.
    ProvisionUpstreams(ProvisionUpstreamsArgs),
    /// Vendor the source-lock-pinned services orderbook OpenAPI document.
    VendorOpenapi(vendor_openapi::VendorOpenApiArgs),
    /// Generate or validate OpenAPI DTO coverage inventories.
    OpenapiCoverage(openapi_coverage::OpenApiCoverageArgs),
    /// Validate every `.sol` file under `crates/contracts/abi/` against
    /// the source-lock-pinned upstream sources. Each file is
    /// SHA-256-checked against the matching `vendored:` row in
    /// `parity/source-lock.yaml`. With `--upstream-root <path>` the
    /// verifier additionally cross-checks against the live upstream
    /// bytes via `git show <commit>:<path>`; with `--upstream-github`
    /// it fetches the bytes from GitHub raw content at the pinned
    /// commit so CI can verify the manifest against canonical upstream
    /// without any local checkout. A provenance-headed excerpt fallback
    /// is recognised for files whose canonical upstream cannot be
    /// vendored as a single byte-stream.
    VerifySolProvenance(verify_sol_provenance::VerifySolProvenanceArgs),
}

#[derive(Debug, Args)]
struct SourceLockArg {
    #[arg(long, default_value = DEFAULT_SOURCE_LOCK)]
    source_lock: PathBuf,
}

#[derive(Debug, Args)]
struct UpstreamRootsArgs {
    #[arg(long, default_value = DEFAULT_SOURCE_LOCK)]
    output: PathBuf,
    #[arg(long)]
    contracts_root: PathBuf,
    #[arg(long)]
    services_root: PathBuf,
}

#[derive(Debug, Args)]
struct ValidateArgs {
    #[command(flatten)]
    source: SourceLockArg,
    #[arg(long)]
    contracts_root: Option<PathBuf>,
    #[arg(long)]
    services_root: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct ProvisionUpstreamsArgs {
    #[command(flatten)]
    source: SourceLockArg,
    #[arg(long)]
    output_root: PathBuf,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Snapshot(args) => snapshot(&CliOptions {
            source_lock: args.output.clone(),
            output: args.output,
            output_root: None,
            contracts_root: Some(args.contracts_root),
            services_root: Some(args.services_root),
        }),
        Commands::Validate(args) => validate(&CliOptions {
            source_lock: args.source.source_lock,
            output: PathBuf::from(DEFAULT_SOURCE_LOCK),
            output_root: None,
            contracts_root: args.contracts_root,
            services_root: args.services_root,
        }),
        Commands::ProvisionUpstreams(args) => provision_upstreams(&CliOptions {
            source_lock: args.source.source_lock,
            output: PathBuf::from(DEFAULT_SOURCE_LOCK),
            output_root: Some(args.output_root),
            contracts_root: None,
            services_root: None,
        }),
        Commands::VendorOpenapi(args) => vendor_openapi::run(args),
        Commands::OpenapiCoverage(args) => openapi_coverage::run(args),
        Commands::VerifySolProvenance(args) => verify_sol_provenance::run(&args),
    }
}

fn snapshot(options: &CliOptions) -> Result<()> {
    let roots = resolve_required_roots(options)?;
    let repositories = REPO_TEMPLATES
        .iter()
        .map(|template| {
            let root = roots
                .get(template.id)
                .with_context(|| format!("missing root for {}", template.id))?;
            build_repository_entry(*template, root)
        })
        .collect::<Result<Vec<_>>>()?;

    let source_lock = SourceLock {
        meta: LockMeta {
            schema_version: 3,
            generated_at_utc: GENERATED_AT_UTC.to_string(),
            purpose: "pinned upstream source contract for committed parity fixtures".to_string(),
        },
        repositories,
        fixtures: fixture_contracts(),
    };

    let yaml = serde_yaml::to_string(&source_lock).context("failed to serialize source lock")?;
    if let Some(parent) = options.output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&options.output, yaml)
        .with_context(|| format!("failed to write {}", options.output.display()))?;

    println!("wrote {}", options.output.display());
    Ok(())
}

fn provision_upstreams(options: &CliOptions) -> Result<()> {
    let lock = load_source_lock(&options.source_lock)?;
    if lock.meta.schema_version != 3 {
        bail!("expected source-lock schema_version 3");
    }

    let output_root = options
        .output_root
        .as_deref()
        .context("provision-upstreams requires --output-root")?;

    if output_root.exists() {
        fs::remove_dir_all(output_root)
            .with_context(|| format!("failed to clear {}", output_root.display()))?;
    }
    fs::create_dir_all(output_root)
        .with_context(|| format!("failed to create {}", output_root.display()))?;

    for repo in &lock.repositories {
        let checkout_root = output_root.join(&repo.id);
        provision_repository_checkout(repo, &checkout_root)?;
    }

    println!(
        "provisioned {} pinned upstream repositories under {}",
        lock.repositories.len(),
        output_root.display()
    );
    Ok(())
}

fn validate(options: &CliOptions) -> Result<()> {
    let lock = load_source_lock(&options.source_lock)?;

    if lock.meta.schema_version != 3 {
        bail!("expected source-lock schema_version 3");
    }

    let helper_mode = lock.repositories.iter().any(|repo| {
        matches!(
            repo.id.as_str(),
            "composable-cow" | "cow-shed" | "watch-tower"
        )
    });
    let expected_source = if helper_mode {
        HELPER_REPO_TEMPLATES
    } else {
        REPO_TEMPLATES
    };
    let expected_templates: BTreeMap<&str, RepoTemplate> = expected_source
        .iter()
        .map(|template| (template.id, *template))
        .collect();
    let dependency_templates: BTreeMap<&str, RepoTemplate> = DEPENDENCY_REPO_TEMPLATES
        .iter()
        .map(|template| (template.id, *template))
        .collect();
    let actual_repos: BTreeMap<&str, &RepositoryEntry> = lock
        .repositories
        .iter()
        .map(|repo| (repo.id.as_str(), repo))
        .collect();

    if actual_repos.len() != lock.repositories.len() {
        bail!("duplicate repository id in source lock");
    }

    for (id, template) in &expected_templates {
        let repo = actual_repos
            .get(id)
            .with_context(|| format!("missing repository entry for {id}"))?;

        validate_repository_entry_template(id, repo, template)?;
        if helper_mode && (repo.pinned_at.is_none() || repo.pinned_by.is_none()) {
            bail!("repository {id} must carry pinned_at and pinned_by metadata");
        }
    }

    for (id, repo) in &actual_repos {
        if expected_templates.contains_key(id) {
            continue;
        }
        let template = dependency_templates
            .get(id)
            .with_context(|| format!("unexpected repository entry in source lock: {id}"))?;
        validate_repository_entry_template(id, repo, template)?;
    }

    let roots = resolve_optional_roots(options);
    for (id, root) in &roots {
        let repo = actual_repos
            .get(id.as_str())
            .with_context(|| format!("missing repository entry for {}", id))?;
        validate_repository_root(repo, root)?;
    }

    let repo_paths: BTreeMap<&str, Vec<&str>> = lock
        .repositories
        .iter()
        .map(|repo| {
            (
                repo.id.as_str(),
                repo.producer_paths.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    let repo_commits: BTreeMap<&str, &str> = lock
        .repositories
        .iter()
        .map(|repo| (repo.id.as_str(), repo.commit.as_str()))
        .collect();

    for fixture in &lock.fixtures {
        if !Path::new(&fixture.file).exists() {
            bail!("fixture file missing: {}", fixture.file);
        }
        let fixture_raw = fs::read_to_string(&fixture.file)
            .with_context(|| format!("failed to read fixture {}", fixture.file))?;
        let fixture_json: serde_json::Value = serde_json::from_str(&fixture_raw)
            .with_context(|| format!("failed to parse fixture {}", fixture.file))?;

        for source_ref in &fixture.source_refs {
            let known_paths = repo_paths
                .get(source_ref.repo.as_str())
                .with_context(|| format!("unknown repo {} in fixture contract", source_ref.repo))?;
            if !known_paths.contains(&source_ref.path.as_str()) {
                bail!(
                    "fixture {} references path not declared in source lock: {}:{}",
                    fixture.file,
                    source_ref.repo,
                    source_ref.path
                );
            }
        }

        if let Some(embedded_source_refs) = fixture_json
            .get("source_refs")
            .and_then(serde_json::Value::as_array)
        {
            for embedded_ref in embedded_source_refs {
                let Some(repo) = embedded_ref.get("repo").and_then(serde_json::Value::as_str)
                else {
                    bail!("fixture {} has source_ref without repo", fixture.file);
                };
                let Some(path) = embedded_ref.get("path").and_then(serde_json::Value::as_str)
                else {
                    bail!("fixture {} has source_ref without path", fixture.file);
                };

                let known_paths = repo_paths
                    .get(repo)
                    .with_context(|| format!("unknown repo {repo} in fixture {}", fixture.file))?;
                if !known_paths.contains(&path) {
                    bail!(
                        "fixture {} embeds source_ref path not declared in source lock: {}:{}",
                        fixture.file,
                        repo,
                        path
                    );
                }

                if let Some(commit) = embedded_ref
                    .get("commit")
                    .and_then(serde_json::Value::as_str)
                {
                    let expected_commit = repo_commits.get(repo).with_context(|| {
                        format!("missing repository commit for embedded source_ref repo {repo}")
                    })?;
                    if commit != *expected_commit {
                        bail!(
                            "fixture {} embeds stale commit for repo {}: fixture={}, lock={}",
                            fixture.file,
                            repo,
                            commit,
                            expected_commit
                        );
                    }
                }
            }
        }
    }

    println!(
        "validated {} repositories and {} fixture contracts",
        lock.repositories.len(),
        lock.fixtures.len()
    );
    Ok(())
}

fn resolve_required_roots(options: &CliOptions) -> Result<BTreeMap<String, PathBuf>> {
    let mut roots = BTreeMap::new();
    roots.insert(
        "contracts".to_string(),
        options
            .contracts_root
            .clone()
            .context("snapshot requires --contracts-root")?,
    );
    roots.insert(
        "services".to_string(),
        options
            .services_root
            .clone()
            .context("snapshot requires --services-root")?,
    );
    Ok(roots)
}

fn resolve_optional_roots(options: &CliOptions) -> BTreeMap<String, PathBuf> {
    let mut roots = BTreeMap::new();
    if let Some(path) = &options.contracts_root {
        roots.insert("contracts".to_string(), path.clone());
    }
    if let Some(path) = &options.services_root {
        roots.insert("services".to_string(), path.clone());
    }
    roots
}

fn load_source_lock(path: &Path) -> Result<SourceLock> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_yaml::from_str(&raw).context("failed to parse source lock")
}

fn validate_repository_entry_template(
    id: &str,
    repo: &RepositoryEntry,
    template: &RepoTemplate,
) -> Result<()> {
    if repo.remote != template.remote {
        bail!("repository {id} remote mismatch");
    }
    if repo.role != template.role {
        bail!("repository {id} role mismatch");
    }
    if repo.optional_local_path != template.local_hint {
        bail!("repository {id} local hint mismatch");
    }

    let expected_paths: Vec<String> = template
        .producer_paths
        .iter()
        .map(|path| (*path).to_string())
        .collect();
    if repo.producer_paths != expected_paths {
        bail!("repository {id} producer paths do not match expected contract");
    }
    Ok(())
}

fn repository_entry<'a>(lock: &'a SourceLock, id: &str) -> Result<&'a RepositoryEntry> {
    lock.repositories
        .iter()
        .find(|repo| repo.id == id)
        .with_context(|| format!("missing repository entry for {id}"))
}

fn validate_repository_root(repo: &RepositoryEntry, root: &Path) -> Result<()> {
    let canonical_root = fs::canonicalize(root)
        .with_context(|| format!("failed to canonicalize {}", root.display()))?;
    validate_git_toplevel(repo, &canonical_root)?;
    validate_repository_remote(repo, &canonical_root)?;

    let commit = git_stdout(&canonical_root, &["rev-parse", "HEAD"])?;
    if repo.commit != commit {
        bail!(
            "repository {} commit mismatch: lock={}, actual={}",
            repo.id,
            repo.commit,
            commit
        );
    }
    for producer_path in &repo.producer_paths {
        let path = canonical_root.join(producer_path);
        if !path.exists() {
            bail!("missing producer path {}", path.display());
        }
    }
    validate_clean_producer_paths(repo, &canonical_root)?;
    Ok(())
}

fn validate_git_toplevel(repo: &RepositoryEntry, root: &Path) -> Result<()> {
    let git_root_raw = git_stdout(root, &["rev-parse", "--show-toplevel"])?;
    let git_root = fs::canonicalize(Path::new(&git_root_raw))
        .with_context(|| format!("failed to canonicalize git top-level {git_root_raw}"))?;

    if git_root != root {
        bail!(
            "repository {} root mismatch: supplied root {} resolves to git top-level {}; supply an independent checkout of {} at the pinned source-lock commit",
            repo.id,
            root.display(),
            git_root.display(),
            repo.remote
        );
    }

    Ok(())
}

fn validate_repository_remote(repo: &RepositoryEntry, root: &Path) -> Result<()> {
    let expected = normalize_repository_remote(&repo.remote);
    let remotes = git_stdout(root, &["remote", "-v"])?;
    let mut found = Vec::new();

    for line in remotes.lines() {
        let mut parts = line.split_whitespace();
        let _name = parts.next();
        if let Some(url) = parts.next() {
            found.push(url.to_string());
        }
    }

    if found
        .iter()
        .any(|remote| normalize_repository_remote(remote) == expected)
    {
        return Ok(());
    }

    let found = if found.is_empty() {
        "none".to_string()
    } else {
        found.join(", ")
    };
    bail!(
        "repository {} remote mismatch: expected {}, found {}",
        repo.id,
        repo.remote,
        found
    );
}

fn validate_clean_producer_paths(repo: &RepositoryEntry, root: &Path) -> Result<()> {
    let mut args: Vec<&str> = vec!["status", "--porcelain", "--"];
    args.extend(repo.producer_paths.iter().map(String::as_str));

    let status = git_stdout(root, &args)?;
    if !status.trim().is_empty() {
        bail!(
            "repository {} has uncommitted changes in producer paths:\n{}",
            repo.id,
            status.trim()
        );
    }

    Ok(())
}

fn normalize_repository_remote(remote: &str) -> String {
    let mut normalized = remote.trim().trim_end_matches('/').to_ascii_lowercase();

    if let Some(rest) = normalized.strip_prefix("git@github.com:") {
        normalized = format!("github.com/{rest}");
    } else if let Some(rest) = normalized.strip_prefix("ssh://git@github.com/") {
        normalized = format!("github.com/{rest}");
    } else if let Some(rest) = normalized.strip_prefix("https://github.com/") {
        normalized = format!("github.com/{rest}");
    }

    normalized
        .strip_suffix(".git")
        .unwrap_or(&normalized)
        .to_string()
}

fn build_repository_entry(template: RepoTemplate, root: &Path) -> Result<RepositoryEntry> {
    let commit = git_stdout(root, &["rev-parse", "HEAD"])?;
    for producer_path in template.producer_paths {
        let path = root.join(producer_path);
        if !path.exists() {
            bail!("missing producer path {}", path.display());
        }
    }

    Ok(RepositoryEntry {
        id: template.id.to_string(),
        remote: template.remote.to_string(),
        commit,
        role: template.role.to_string(),
        optional_local_path: template.local_hint.to_string(),
        producer_paths: template
            .producer_paths
            .iter()
            .map(|path| (*path).to_string())
            .collect(),
        pinned_at: None,
        pinned_by: None,
    })
}

fn provision_repository_checkout(repo: &RepositoryEntry, checkout_root: &Path) -> Result<()> {
    if checkout_root.exists() {
        fs::remove_dir_all(checkout_root)
            .with_context(|| format!("failed to clear {}", checkout_root.display()))?;
    }

    let parent = checkout_root
        .parent()
        .with_context(|| format!("missing parent for {}", checkout_root.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    let checkout_name = checkout_root
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("invalid checkout path {}", checkout_root.display()))?;
    run_git_command(
        parent,
        &[
            "clone",
            "--filter=blob:none",
            "--no-checkout",
            repo.remote.as_str(),
            checkout_name,
        ],
    )?;

    if let Err(error) = run_git_command(
        checkout_root,
        &["fetch", "--depth", "1", "origin", repo.commit.as_str()],
    ) {
        eprintln!(
            "shallow fetch failed for {} at {}: {error:#}; retrying with a full commit fetch",
            repo.id, repo.commit
        );
        run_git_command(checkout_root, &["fetch", "origin", repo.commit.as_str()])?;
    }

    run_git_command(
        checkout_root,
        &["checkout", "--detach", repo.commit.as_str()],
    )?;
    validate_repository_root(repo, checkout_root)?;
    Ok(())
}

fn run_git_command(root: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git in {}", root.display()))?;
    if !output.status.success() {
        bail!(
            "git command failed in {}: {}",
            root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn git_stdout(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git in {}", root.display()))?;
    if !output.status.success() {
        bail!(
            "git command failed in {}: {}",
            root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn fixture_contracts() -> Vec<FixtureEntry> {
    vec![
        FixtureEntry {
            surface: "contracts".to_string(),
            file: "parity/fixtures/contracts.json".to_string(),
            source_refs: vec![
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "src/ts/order.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "src/ts/sign.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "src/ts/settlement.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "src/ts/swap.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "src/ts/interaction.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "src/ts/vault.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "src/ts/proxy.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "test/GPv2Order/PackOrderUidParams.t.sol".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "test/GPv2Order/ExtractOrderUidParams.t.sol".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "test/GPv2Trade/ExtractFlags.t.sol".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "test/GPv2Trade/ExtractOrder.t.sol".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "test/GPv2Settlement/OrderRefunds.t.sol".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "test/GPv2Settlement/Swap/Swap.t.sol".to_string(),
                },
            ],
        },
        FixtureEntry {
            surface: "trading".to_string(),
            file: "parity/fixtures/trading.json".to_string(),
            source_refs: vec![
                FixtureSourceRef {
                    repo: "services".to_string(),
                    path: "crates/orderbook/openapi.yml".to_string(),
                },
                FixtureSourceRef {
                    repo: "services".to_string(),
                    path: "crates/orderbook/src/quoter.rs".to_string(),
                },
                FixtureSourceRef {
                    repo: "services".to_string(),
                    path: "crates/shared/src/order_validation.rs".to_string(),
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        sync::{Mutex, OnceLock},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn cwd_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct CwdGuard {
        original: PathBuf,
    }

    impl CwdGuard {
        fn change_to(path: &Path) -> Result<Self> {
            let original = env::current_dir().context("failed to read current dir")?;
            env::set_current_dir(path)
                .with_context(|| format!("failed to change current dir to {}", path.display()))?;
            Ok(Self { original })
        }
    }

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.original);
        }
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        env::temp_dir().join(format!(
            "parity-maintainer-{name}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn run_git(root: &Path, args: &[&str]) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .with_context(|| format!("failed to run git in {}", root.display()))?;
        if !output.status.success() {
            bail!(
                "git command failed in {}: {}",
                root.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(())
    }

    fn write_producer_paths(root: &Path, producer_paths: &[&str]) -> Result<()> {
        fs::create_dir_all(root).with_context(|| format!("failed to create {}", root.display()))?;
        for producer_path in producer_paths {
            let path = root.join(producer_path);
            let parent = path
                .parent()
                .with_context(|| format!("missing parent for {}", path.display()))?;
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
            fs::write(&path, format!("fixture source for {producer_path}\n"))
                .with_context(|| format!("failed to write {}", path.display()))?;
        }
        Ok(())
    }

    fn init_repo(root: &Path, producer_paths: &[&str], remote: &str) -> Result<String> {
        write_producer_paths(root, producer_paths)?;
        run_git(root, &["init"])?;
        run_git(root, &["config", "user.email", "tests@example.com"])?;
        run_git(root, &["config", "user.name", "Parity Maintainer Tests"])?;
        run_git(root, &["remote", "add", "origin", remote])?;

        run_git(root, &["add", "."])?;
        run_git(root, &["commit", "-m", "initial fixture sources"])?;
        git_stdout(root, &["rev-parse", "HEAD"])
    }

    fn repository_entry_for(template: RepoTemplate, commit: &str) -> RepositoryEntry {
        RepositoryEntry {
            id: template.id.to_string(),
            remote: template.remote.to_string(),
            commit: commit.to_string(),
            role: template.role.to_string(),
            optional_local_path: template.local_hint.to_string(),
            producer_paths: template
                .producer_paths
                .iter()
                .map(|path| (*path).to_string())
                .collect(),
            pinned_at: None,
            pinned_by: None,
        }
    }

    fn write_fixture_files(
        workspace_root: &Path,
        repo_commits: &BTreeMap<String, String>,
    ) -> Result<()> {
        for fixture in fixture_contracts() {
            let path = workspace_root.join(&fixture.file);
            let parent = path
                .parent()
                .with_context(|| format!("missing parent for {}", path.display()))?;
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
            let fixture_json = serde_json::json!({
                "schema_version": 1,
                "surface": fixture.surface,
                "source_refs": fixture.source_refs.iter().map(|source_ref| {
                    serde_json::json!({
                        "repo": source_ref.repo,
                        "commit": repo_commits
                            .get(&source_ref.repo)
                            .expect("missing repo commit for fixture source ref"),
                        "path": source_ref.path
                    })
                }).collect::<Vec<_>>()
            });
            fs::write(
                &path,
                serde_json::to_string_pretty(&fixture_json)
                    .context("failed to serialize fixture json")?,
            )
            .with_context(|| format!("failed to write {}", path.display()))?;
        }
        Ok(())
    }

    struct TestWorkspace {
        root: PathBuf,
        contracts_root: PathBuf,
        services_root: PathBuf,
    }

    impl TestWorkspace {
        fn new(name: &str) -> Result<Self> {
            let root = unique_temp_dir(name);
            fs::create_dir_all(&root)
                .with_context(|| format!("failed to create {}", root.display()))?;

            let contracts_root = root.join("contracts");
            let services_root = root.join("services");

            let contracts_commit =
                init_repo(&contracts_root, CONTRACTS_PATHS, REPO_TEMPLATES[0].remote)?;
            let services_commit =
                init_repo(&services_root, SERVICES_PATHS, REPO_TEMPLATES[1].remote)?;

            let repo_commits = BTreeMap::from([
                ("contracts".to_string(), contracts_commit),
                ("services".to_string(), services_commit),
            ]);
            write_fixture_files(&root, &repo_commits)?;

            Ok(Self {
                root,
                contracts_root,
                services_root,
            })
        }

        fn cli_options(&self) -> CliOptions {
            CliOptions {
                source_lock: self.root.join("parity/source-lock.yaml"),
                output: self.root.join("parity/source-lock.yaml"),
                output_root: None,
                contracts_root: Some(self.contracts_root.clone()),
                services_root: Some(self.services_root.clone()),
            }
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn snapshot_and_validate_round_trip_with_real_roots() -> Result<()> {
        let _lock = cwd_lock().lock().expect("cwd lock poisoned");
        let workspace = TestWorkspace::new("round-trip")?;
        let _guard = CwdGuard::change_to(&workspace.root)?;
        let options = workspace.cli_options();

        snapshot(&options)?;
        validate(&options)?;

        Ok(())
    }

    #[test]
    fn validate_with_roots_rejects_path_that_resolves_to_parent_repo() -> Result<()> {
        let root = unique_temp_dir("parent-repo-resolution");
        let parent = root.join("cow-rs");
        let parent_commit = init_repo(
            &parent,
            &["README.md"],
            "https://github.com/example/cow-rs.git",
        )?;
        let nested_contracts = parent.join("copied-upstream/contracts");
        write_producer_paths(&nested_contracts, CONTRACTS_PATHS)?;

        let contracts_repo = repository_entry_for(REPO_TEMPLATES[0], &parent_commit);
        let error = validate_repository_root(&contracts_repo, &nested_contracts)
            .expect_err("validate should fail when the supplied root resolves to a parent repo");

        assert!(
            format!("{error:#}").contains("root mismatch"),
            "unexpected error: {error:#}"
        );
        assert!(
            format!("{error:#}").contains("supply an independent checkout"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn validate_with_roots_rejects_wrong_remote() -> Result<()> {
        let _lock = cwd_lock().lock().expect("cwd lock poisoned");
        let workspace = TestWorkspace::new("wrong-remote")?;
        let _guard = CwdGuard::change_to(&workspace.root)?;
        let options = workspace.cli_options();

        snapshot(&options)?;
        run_git(
            &workspace.contracts_root,
            &[
                "remote",
                "set-url",
                "origin",
                "https://github.com/example/contracts.git",
            ],
        )?;

        let error = validate(&options).expect_err("validate should fail on remote mismatch");
        assert!(
            format!("{error:#}").contains("repository contracts remote mismatch"),
            "unexpected error: {error:#}"
        );

        Ok(())
    }

    #[test]
    fn validate_with_roots_accepts_ssh_github_remote_for_expected_repo() -> Result<()> {
        let _lock = cwd_lock().lock().expect("cwd lock poisoned");
        let workspace = TestWorkspace::new("ssh-remote")?;
        let _guard = CwdGuard::change_to(&workspace.root)?;
        let options = workspace.cli_options();

        snapshot(&options)?;
        run_git(
            &workspace.contracts_root,
            &[
                "remote",
                "set-url",
                "origin",
                "git@github.com:cowprotocol/contracts.git",
            ],
        )?;

        validate(&options)?;

        Ok(())
    }

    #[test]
    fn validate_with_roots_rejects_dirty_producer_paths() -> Result<()> {
        let _lock = cwd_lock().lock().expect("cwd lock poisoned");
        let workspace = TestWorkspace::new("dirty-producer")?;
        let _guard = CwdGuard::change_to(&workspace.root)?;
        let options = workspace.cli_options();

        snapshot(&options)?;
        fs::write(
            workspace.contracts_root.join("src/ts/order.ts"),
            "local uncommitted producer drift\n",
        )
        .context("failed to dirty contracts producer path")?;

        let error = validate(&options).expect_err("validate should fail on dirty producer paths");
        assert!(
            format!("{error:#}")
                .contains("repository contracts has uncommitted changes in producer paths"),
            "unexpected error: {error:#}"
        );

        Ok(())
    }

    #[test]
    fn validate_with_roots_rejects_commit_mismatch() -> Result<()> {
        let _lock = cwd_lock().lock().expect("cwd lock poisoned");
        let workspace = TestWorkspace::new("commit-mismatch")?;
        let _guard = CwdGuard::change_to(&workspace.root)?;
        let options = workspace.cli_options();

        snapshot(&options)?;

        let mut lock: SourceLock = serde_yaml::from_str(
            &fs::read_to_string(&options.source_lock)
                .with_context(|| format!("failed to read {}", options.source_lock.display()))?,
        )
        .context("failed to parse generated source lock")?;
        let contracts_entry = lock
            .repositories
            .iter_mut()
            .find(|repo| repo.id == "contracts")
            .context("missing contracts entry in generated source lock")?;
        contracts_entry.commit = "0000000000000000000000000000000000000000".to_string();
        fs::write(
            &options.source_lock,
            serde_yaml::to_string(&lock).context("failed to serialize mutated source lock")?,
        )
        .with_context(|| format!("failed to write {}", options.source_lock.display()))?;

        let error = validate(&options).expect_err("validate should fail on commit mismatch");
        assert!(
            format!("{error:#}").contains("repository contracts commit mismatch"),
            "unexpected error: {error:#}"
        );

        Ok(())
    }

    #[test]
    fn validate_without_roots_still_checks_internal_contract() -> Result<()> {
        let _lock = cwd_lock().lock().expect("cwd lock poisoned");
        let workspace = TestWorkspace::new("standalone-validate")?;
        let _guard = CwdGuard::change_to(&workspace.root)?;
        let options = workspace.cli_options();

        snapshot(&options)?;

        let mut lock: SourceLock = serde_yaml::from_str(
            &fs::read_to_string(&options.source_lock)
                .with_context(|| format!("failed to read {}", options.source_lock.display()))?,
        )
        .context("failed to parse generated source lock")?;
        let contracts_entry = lock
            .repositories
            .iter_mut()
            .find(|repo| repo.id == "contracts")
            .context("missing contracts entry in generated source lock")?;
        contracts_entry.commit = "1111111111111111111111111111111111111111".to_string();
        fs::write(
            &options.source_lock,
            serde_yaml::to_string(&lock).context("failed to serialize mutated source lock")?,
        )
        .with_context(|| format!("failed to write {}", options.source_lock.display()))?;

        let fixture_path = workspace.root.join("parity/fixtures/contracts.json");
        let mut fixture_json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&fixture_path)
                .with_context(|| format!("failed to read {}", fixture_path.display()))?,
        )
        .context("failed to parse contracts fixture json")?;
        let source_refs = fixture_json["source_refs"]
            .as_array_mut()
            .context("missing contracts source_refs array")?;
        for source_ref in source_refs {
            if source_ref["repo"].as_str() == Some("contracts") {
                source_ref["commit"] = serde_json::Value::String(
                    "1111111111111111111111111111111111111111".to_string(),
                );
            }
        }
        fs::write(
            &fixture_path,
            serde_json::to_string_pretty(&fixture_json)
                .context("failed to serialize orderbook fixture")?,
        )
        .with_context(|| format!("failed to write {}", fixture_path.display()))?;

        let standalone = CliOptions {
            source_lock: options.source_lock.clone(),
            output: options.output.clone(),
            output_root: None,
            contracts_root: None,
            services_root: None,
        };

        validate(&standalone)?;

        Ok(())
    }

    #[test]
    fn validate_rejects_fixture_commit_drift_without_roots() -> Result<()> {
        let _lock = cwd_lock().lock().expect("cwd lock poisoned");
        let workspace = TestWorkspace::new("fixture-commit-drift")?;
        let _guard = CwdGuard::change_to(&workspace.root)?;
        let options = workspace.cli_options();

        snapshot(&options)?;

        let fixture_path = workspace.root.join("parity/fixtures/trading.json");
        let mut fixture_json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&fixture_path)
                .with_context(|| format!("failed to read {}", fixture_path.display()))?,
        )
        .context("failed to parse fixture json")?;
        let trading_source_refs = fixture_json["source_refs"]
            .as_array_mut()
            .context("missing trading source_refs array")?;
        let services_ref = trading_source_refs
            .iter_mut()
            .find(|source_ref| source_ref["repo"].as_str() == Some("services"))
            .context("missing services source_ref in trading fixture")?;
        services_ref["commit"] =
            serde_json::Value::String("2222222222222222222222222222222222222222".to_string());
        fs::write(
            &fixture_path,
            serde_json::to_string_pretty(&fixture_json).context("failed to serialize fixture")?,
        )
        .with_context(|| format!("failed to write {}", fixture_path.display()))?;

        let standalone = CliOptions {
            source_lock: options.source_lock.clone(),
            output: options.output.clone(),
            output_root: None,
            contracts_root: None,
            services_root: None,
        };

        let error =
            validate(&standalone).expect_err("validate should fail on fixture commit drift");
        assert!(
            format!("{error:#}").contains("embeds stale commit for repo services"),
            "unexpected error: {error:#}"
        );

        Ok(())
    }
}
