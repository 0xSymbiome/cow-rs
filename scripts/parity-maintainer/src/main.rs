use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

const GENERATED_AT_UTC: &str = "2026-04-08T00:00:00Z";
const DEFAULT_SOURCE_LOCK: &str = "parity/source-lock.yaml";
const APP_DATA_SCHEMA_SOURCE_DIR: &str = "packages/app-data/src/schemas";
const APP_DATA_SCHEMA_VENDOR_DIR: &str = "crates/app-data/schemas";

#[derive(Clone, Copy)]
struct RepoTemplate {
    id: &'static str,
    remote: &'static str,
    role: &'static str,
    local_hint: &'static str,
    producer_paths: &'static [&'static str],
}

const COW_SDK_PATHS: &[&str] = &[
    "packages/common/src/adapters/AbstractSigner.ts",
    "packages/common/src/adapters/AbstractProviderAdapter.ts",
    "packages/common/src/utils/address.ts",
    "packages/common/src/utils/address.test.ts",
    "packages/common/src/utils/token.ts",
    "packages/common/src/utils/token.test.ts",
    "packages/config/src/types/configs.ts",
    "packages/config/src/chains/types.ts",
    "packages/config/src/constants/addresses.ts",
    "packages/config/src/constants/wrappedTokens.ts",
    "packages/trading/src/index.ts",
    "packages/trading/src/types.ts",
    "packages/trading/src/appDataUtils.ts",
    "packages/trading/src/appDataUtils.test.ts",
    "packages/trading/src/getQuote.ts",
    "packages/trading/src/getQuote.test.ts",
    "packages/trading/src/getOrderToSign.ts",
    "packages/trading/src/getOrderToSign.test.ts",
    "packages/trading/src/getOrderTypedData.ts",
    "packages/trading/src/getOrderTypedData.test.ts",
    "packages/trading/src/calculateUniqueOrderId.ts",
    "packages/trading/src/calculateUniqueOrderId.test.ts",
    "packages/trading/src/getSettlementContract.ts",
    "packages/trading/src/getSettlementContract.test.ts",
    "packages/trading/src/getPreSignTransaction.ts",
    "packages/trading/src/getPreSignTransaction.test.ts",
    "packages/trading/src/getEthFlowTransaction.ts",
    "packages/trading/src/getEthFlowTransaction.test.ts",
    "packages/trading/src/onChainCancellation.ts",
    "packages/trading/src/onChainCancellation.test.ts",
    "packages/trading/src/postCoWProtocolTrade.ts",
    "packages/trading/src/postCoWProtocolTrade.test.ts",
    "packages/trading/src/postLimitOrder.ts",
    "packages/trading/src/postLimitOrder.test.ts",
    "packages/trading/src/postSellNativeCurrencyOrder.ts",
    "packages/trading/src/postSellNativeCurrencyOrder.test.ts",
    "packages/trading/src/postSwapOrder.ts",
    "packages/trading/src/postSwapOrder.test.ts",
    "packages/trading/src/resolveSlippageSuggestion.ts",
    "packages/trading/src/resolveSlippageSuggestion.test.ts",
    "packages/trading/src/suggestSlippageBps.ts",
    "packages/trading/src/suggestSlippageBps.test.ts",
    "packages/trading/src/suggestSlippageFromFee.ts",
    "packages/trading/src/suggestSlippageFromFee.test.ts",
    "packages/trading/src/suggestSlippageFromVolume.ts",
    "packages/trading/src/suggestSlippageFromVolume.test.ts",
    "packages/trading/src/tradingSdk.ts",
    "packages/trading/src/tradingSdk.test.ts",
    "packages/order-signing/src/orderSigningUtils.ts",
    "packages/order-signing/src/utils.ts",
    "packages/order-signing/src/types.ts",
    "packages/order-signing/tests/orderSigningUtils.test.ts",
    "packages/order-book/src/api.ts",
    "packages/order-book/src/api.spec.ts",
    "packages/order-book/src/request.ts",
    "packages/order-book/src/request.test.ts",
    "packages/order-book/src/transformOrder.ts",
    "packages/order-book/src/transformOrder.test.ts",
    "packages/order-book/src/types.ts",
    "packages/contracts-ts/src/ContractsTs.ts",
    "packages/contracts-ts/src/types.ts",
    "packages/contracts-ts/src/order.ts",
    "packages/contracts-ts/src/sign.ts",
    "packages/contracts-ts/src/settlement.ts",
    "packages/contracts-ts/src/swap.ts",
    "packages/contracts-ts/src/interaction.ts",
    "packages/contracts-ts/src/proxy.ts",
    "packages/contracts-ts/src/vault.ts",
    "packages/contracts-ts/src/reader.ts",
    "packages/contracts-ts/tests/order.test.ts",
    "packages/contracts-ts/tests/order-processing.test.ts",
    "packages/contracts-ts/tests/settlement.test.ts",
    "packages/contracts-ts/tests/signature.test.ts",
    "packages/contracts-ts/tests/deployment.test.ts",
    "packages/contracts-ts/tests/vault.test.ts",
    "packages/contracts-ts/tests/reader.test.ts",
    "packages/app-data/src/api/appDataHexToCid.ts",
    "packages/app-data/src/mocks.ts",
    "packages/app-data/src/api/appDataHexToCid.spec.ts",
    "packages/app-data/src/api/cidToAppDataHex.ts",
    "packages/app-data/src/api/cidToAppDataHex.test.ts",
    "packages/app-data/src/api/generateAppDataDoc.ts",
    "packages/app-data/src/api/generateAppDataDoc.spec.ts",
    "packages/app-data/src/api/getAppDataInfo.ts",
    "packages/app-data/src/api/getAppDataInfo.spec.ts",
    "packages/app-data/src/api/getAppDataSchema.ts",
    "packages/app-data/src/api/getAppDataSchema.spec.ts",
    "packages/app-data/src/api/validateAppDataDoc.ts",
    "packages/app-data/src/api/validateAppDataDoc.spec.ts",
    "packages/app-data/src/api/fetchDocFromCid.ts",
    "packages/app-data/src/api/fetchDocFromCid.spec.ts",
    "packages/app-data/src/api/fetchDocFromAppData.ts",
    "packages/app-data/src/api/fetchDocFromAppData.spec.ts",
    "packages/app-data/src/api/uploadMetadataDocToIpfsLegacy.ts",
    "packages/app-data/src/api/uploadMetadataDocToIpfsLegacy.spec.ts",
    "packages/app-data/src/types.ts",
    "packages/app-data/src/consts.ts",
    "packages/app-data/src/importSchema.ts",
    "packages/app-data/src/utils/ipfs.ts",
    "packages/app-data/src/utils/stringify.ts",
    "packages/app-data/src/generatedTypes/index.ts",
    "packages/app-data/src/generatedTypes/latest.ts",
    "packages/app-data/test/schema.spec.ts",
    "packages/app-data/test/schema-1.5.0.spec.ts",
    "packages/app-data/test/flashloan-v1.7.0.spec.ts",
    "packages/app-data/test/wrappers.v1.13.0.spec.ts",
    "packages/subgraph/src/api.ts",
    "packages/subgraph/src/api.spec.ts",
    "packages/subgraph/src/graphql.ts",
    "packages/subgraph/src/queries.ts",
    "packages/sdk/src/index.ts",
    "packages/sdk/src/typedoc-entry.ts",
    "packages/sdk/package.json",
    "packages/sdk/README.md",
];

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
];

const REPO_TEMPLATES: &[RepoTemplate] = &[
    RepoTemplate {
        id: "cow-sdk",
        remote: "https://github.com/cowprotocol/cow-sdk.git",
        role: "primary",
        local_hint: "cow-protocol/cow-sdk",
        producer_paths: COW_SDK_PATHS,
    },
    RepoTemplate {
        id: "contracts",
        remote: "https://github.com/cowprotocol/contracts.git",
        role: "primary",
        local_hint: "cow-protocol/contracts",
        producer_paths: CONTRACTS_PATHS,
    },
    RepoTemplate {
        id: "services",
        remote: "https://github.com/cowprotocol/services.git",
        role: "reference-only",
        local_hint: "cow-protocol/services",
        producer_paths: SERVICES_PATHS,
    },
];

#[derive(Debug, Serialize, Deserialize)]
struct SourceLock {
    meta: LockMeta,
    repositories: Vec<RepositoryEntry>,
    fixtures: Vec<FixtureEntry>,
    validation: ValidationEntry,
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

#[derive(Debug, Serialize, Deserialize)]
struct ValidationEntry {
    standalone_repo_contract: Vec<String>,
    maintainer_refresh_contract: Vec<String>,
}

struct CliOptions {
    source_lock: PathBuf,
    output: PathBuf,
    cow_sdk_root: Option<PathBuf>,
    contracts_root: Option<PathBuf>,
    services_root: Option<PathBuf>,
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        bail!("missing command");
    };

    let options = parse_options(args.collect())?;

    match command.as_str() {
        "snapshot" => snapshot(&options),
        "validate" => validate(&options),
        "vendor-app-data-schemas" => vendor_app_data_schemas(&options),
        _ => {
            print_usage();
            bail!("unknown command: {command}");
        }
    }
}

fn print_usage() {
    eprintln!(
        "usage:\n  cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- snapshot --cow-sdk-root <path> --contracts-root <path> --services-root <path> [--output parity/source-lock.yaml]\n  cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate [--source-lock parity/source-lock.yaml] [--cow-sdk-root <path>] [--contracts-root <path>] [--services-root <path>]\n  cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- vendor-app-data-schemas [--source-lock parity/source-lock.yaml] --cow-sdk-root <real-cow-sdk-clone>"
    );
}

fn parse_options(args: Vec<String>) -> Result<CliOptions> {
    let mut source_lock = PathBuf::from(DEFAULT_SOURCE_LOCK);
    let mut output = PathBuf::from(DEFAULT_SOURCE_LOCK);
    let mut cow_sdk_root = None;
    let mut contracts_root = None;
    let mut services_root = None;

    let mut i = 0;
    while i < args.len() {
        let key = args[i].as_str();
        let value = args
            .get(i + 1)
            .with_context(|| format!("missing value for {key}"))?;

        match key {
            "--source-lock" => source_lock = PathBuf::from(value),
            "--output" => output = PathBuf::from(value),
            "--cow-sdk-root" => cow_sdk_root = Some(PathBuf::from(value)),
            "--contracts-root" => contracts_root = Some(PathBuf::from(value)),
            "--services-root" => services_root = Some(PathBuf::from(value)),
            _ => bail!("unknown option: {key}"),
        }

        i += 2;
    }

    Ok(CliOptions {
        source_lock,
        output,
        cow_sdk_root,
        contracts_root,
        services_root,
    })
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
            schema_version: 2,
            generated_at_utc: GENERATED_AT_UTC.to_string(),
            purpose: "pinned upstream source contract for committed parity fixtures".to_string(),
        },
        repositories,
        fixtures: fixture_contracts(),
        validation: ValidationEntry {
            standalone_repo_contract: vec![
                "cargo build --workspace".to_string(),
                "cargo test --workspace".to_string(),
            ],
            maintainer_refresh_contract: vec![
                "cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml --cow-sdk-root <path> --contracts-root <path> --services-root <path>".to_string(),
                "cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- snapshot --output parity/source-lock.yaml --cow-sdk-root <path> --contracts-root <path> --services-root <path>".to_string(),
                "cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- vendor-app-data-schemas --source-lock parity/source-lock.yaml --cow-sdk-root <path>".to_string(),
            ],
        },
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

fn vendor_app_data_schemas(options: &CliOptions) -> Result<()> {
    let lock = load_source_lock(&options.source_lock)?;
    let cow_sdk_repo = repository_entry(&lock, "cow-sdk")?;
    let cow_sdk_root = options
        .cow_sdk_root
        .as_deref()
        .context("vendor-app-data-schemas requires --cow-sdk-root")?;

    validate_repository_root(cow_sdk_repo, cow_sdk_root)?;

    let source = cow_sdk_root.join(APP_DATA_SCHEMA_SOURCE_DIR);
    let dest = PathBuf::from(APP_DATA_SCHEMA_VENDOR_DIR);
    let source_count =
        validate_schema_bundle_dir(&source, "upstream cow-sdk app-data schema bundle")?;

    sync_directory_tree(&source, &dest)?;
    let copied_count = ensure_matching_file_trees(
        &source,
        &dest,
        "upstream cow-sdk app-data schema bundle",
        "vendored cow-rs app-data schema bundle",
    )?;

    println!(
        "vendored {} app-data schema files from {} at commit {} into {}",
        copied_count,
        source.display(),
        cow_sdk_repo.commit,
        dest.display()
    );

    if copied_count != source_count {
        bail!(
            "schema vendor count mismatch after sync: source={}, copied={}",
            source_count,
            copied_count
        );
    }

    Ok(())
}

fn validate(options: &CliOptions) -> Result<()> {
    let lock = load_source_lock(&options.source_lock)?;

    if lock.meta.schema_version != 2 {
        bail!("expected source-lock schema_version 2");
    }

    let expected_templates: BTreeMap<&str, RepoTemplate> = REPO_TEMPLATES
        .iter()
        .map(|template| (template.id, *template))
        .collect();
    let actual_repos: BTreeMap<&str, &RepositoryEntry> = lock
        .repositories
        .iter()
        .map(|repo| (repo.id.as_str(), repo))
        .collect();

    if actual_repos.len() != expected_templates.len() {
        bail!("unexpected repository count in source lock");
    }

    for (id, template) in &expected_templates {
        let repo = actual_repos
            .get(id)
            .with_context(|| format!("missing repository entry for {id}"))?;

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

    let vendored_schema_count = validate_schema_bundle_dir(
        Path::new(APP_DATA_SCHEMA_VENDOR_DIR),
        "vendored cow-rs app-data schema bundle",
    )?;

    if let Some(cow_sdk_root) = roots.get("cow-sdk") {
        let source = cow_sdk_root.join(APP_DATA_SCHEMA_SOURCE_DIR);
        ensure_matching_file_trees(
            &source,
            Path::new(APP_DATA_SCHEMA_VENDOR_DIR),
            "upstream cow-sdk app-data schema bundle",
            "vendored cow-rs app-data schema bundle",
        )?;
    }

    println!(
        "validated {} repositories, {} fixture contracts, and {} vendored app-data schema files",
        lock.repositories.len(),
        lock.fixtures.len(),
        vendored_schema_count
    );
    Ok(())
}

fn resolve_required_roots(options: &CliOptions) -> Result<BTreeMap<String, PathBuf>> {
    let mut roots = BTreeMap::new();
    roots.insert(
        "cow-sdk".to_string(),
        options
            .cow_sdk_root
            .clone()
            .context("snapshot requires --cow-sdk-root")?,
    );
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
    if let Some(path) = &options.cow_sdk_root {
        roots.insert("cow-sdk".to_string(), path.clone());
    }
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

fn repository_entry<'a>(lock: &'a SourceLock, id: &str) -> Result<&'a RepositoryEntry> {
    lock.repositories
        .iter()
        .find(|repo| repo.id == id)
        .with_context(|| format!("missing repository entry for {id}"))
}

fn validate_repository_root(repo: &RepositoryEntry, root: &Path) -> Result<()> {
    let commit = git_stdout(root, &["rev-parse", "HEAD"])?;
    if repo.commit != commit {
        bail!(
            "repository {} commit mismatch: lock={}, actual={}",
            repo.id,
            repo.commit,
            commit
        );
    }
    for producer_path in &repo.producer_paths {
        let path = root.join(producer_path);
        if !path.exists() {
            bail!("missing producer path {}", path.display());
        }
    }
    Ok(())
}

fn validate_schema_bundle_dir(dir: &Path, label: &str) -> Result<usize> {
    if !dir.exists() {
        bail!("{label} missing: {}", dir.display());
    }
    if !dir.is_dir() {
        bail!("{label} is not a directory: {}", dir.display());
    }

    let files = collect_relative_files(dir)?;
    if files.is_empty() {
        bail!("{label} is empty: {}", dir.display());
    }

    for relative in files.keys() {
        if !relative.ends_with(".json") {
            bail!(
                "{label} contains non-json file {}. crate app-data embeds every file in this tree as schema json",
                relative
            );
        }
    }

    Ok(files.len())
}

fn sync_directory_tree(source: &Path, dest: &Path) -> Result<()> {
    let source_files = collect_relative_files(source)?;
    let staging = dest.with_extension("tmp");

    if staging.exists() {
        fs::remove_dir_all(&staging)
            .with_context(|| format!("failed to clear staging dir {}", staging.display()))?;
    }
    fs::create_dir_all(&staging)
        .with_context(|| format!("failed to create staging dir {}", staging.display()))?;

    for (relative, source_path) in &source_files {
        let target = staging.join(relative);
        let parent = target
            .parent()
            .with_context(|| format!("missing parent for {}", target.display()))?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
        fs::copy(source_path, &target).with_context(|| {
            format!(
                "failed to copy {} -> {}",
                source_path.display(),
                target.display()
            )
        })?;
    }

    if dest.exists() {
        fs::remove_dir_all(dest).with_context(|| format!("failed to clear {}", dest.display()))?;
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    fs::rename(&staging, dest).with_context(|| {
        format!(
            "failed to rename {} -> {}",
            staging.display(),
            dest.display()
        )
    })
}

fn ensure_matching_file_trees(
    source: &Path,
    dest: &Path,
    source_label: &str,
    dest_label: &str,
) -> Result<usize> {
    let source_count = validate_schema_bundle_dir(source, source_label)?;
    let dest_files = collect_relative_files(dest)?;
    let source_files = collect_relative_files(source)?;

    let mut missing = Vec::new();
    let mut extra = Vec::new();
    let mut mismatched = Vec::new();

    for (relative, source_path) in &source_files {
        let Some(dest_path) = dest_files.get(relative) else {
            missing.push(relative.clone());
            continue;
        };

        let source_bytes = fs::read(source_path)
            .with_context(|| format!("failed to read {}", source_path.display()))?;
        let dest_bytes = fs::read(dest_path)
            .with_context(|| format!("failed to read {}", dest_path.display()))?;
        if source_bytes != dest_bytes {
            mismatched.push(relative.clone());
        }
    }

    for relative in dest_files.keys() {
        if !source_files.contains_key(relative) {
            extra.push(relative.clone());
        }
    }

    if !(missing.is_empty() && extra.is_empty() && mismatched.is_empty()) {
        let mut message = format!("{} does not match {}", dest_label, source_label);
        if !missing.is_empty() {
            message.push_str(&format!(
                "\nmissing files in {}: {}",
                dest_label,
                missing.join(", ")
            ));
        }
        if !extra.is_empty() {
            message.push_str(&format!(
                "\nextra files in {}: {}",
                dest_label,
                extra.join(", ")
            ));
        }
        if !mismatched.is_empty() {
            message.push_str(&format!(
                "\ncontent mismatches between {} and {}: {}",
                source_label,
                dest_label,
                mismatched.join(", ")
            ));
        }
        bail!(message);
    }

    Ok(source_count)
}

fn collect_relative_files(root: &Path) -> Result<BTreeMap<String, PathBuf>> {
    let canonical_root = fs::canonicalize(root)
        .with_context(|| format!("failed to canonicalize {}", root.display()))?;
    let mut files = BTreeMap::new();
    collect_relative_files_inner(&canonical_root, &canonical_root, &mut files)?;
    Ok(files)
}

fn collect_relative_files_inner(
    root: &Path,
    current: &Path,
    files: &mut BTreeMap<String, PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(current)
        .with_context(|| format!("failed to read directory {}", current.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to inspect entry in {}", current.display()))?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .with_context(|| format!("failed to read metadata for {}", path.display()))?;
        if metadata.is_dir() {
            collect_relative_files_inner(root, &path, files)?;
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .with_context(|| format!("failed to relativize {}", path.display()))?
                .to_string_lossy()
                .replace('\\', "/");
            files.insert(relative, path);
        }
    }

    Ok(())
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
    })
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
            surface: "core".to_string(),
            file: "parity/fixtures/core.json".to_string(),
            source_refs: vec![
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/common/src/adapters/AbstractSigner.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/common/src/adapters/AbstractProviderAdapter.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/common/src/utils/address.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/common/src/utils/address.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/common/src/utils/token.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/common/src/utils/token.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/config/src/types/configs.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/config/src/chains/types.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/config/src/constants/addresses.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/config/src/constants/wrappedTokens.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/order-signing/src/types.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/order-book/src/types.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "src/ts/order.ts".to_string(),
                },
            ],
        },
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
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/src/order.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/src/sign.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/src/settlement.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/src/swap.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/src/interaction.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/src/proxy.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/src/vault.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/src/reader.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/tests/order.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/tests/order-processing.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/tests/settlement.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/tests/signature.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/tests/deployment.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/tests/vault.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/contracts-ts/tests/reader.test.ts".to_string(),
                },
            ],
        },
        FixtureEntry {
            surface: "signing".to_string(),
            file: "parity/fixtures/signing.json".to_string(),
            source_refs: vec![
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/order-signing/src/orderSigningUtils.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/order-signing/src/utils.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/order-signing/src/types.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/order-signing/tests/orderSigningUtils.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/getOrderTypedData.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/getOrderTypedData.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/calculateUniqueOrderId.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/calculateUniqueOrderId.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "src/ts/order.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "contracts".to_string(),
                    path: "src/ts/sign.ts".to_string(),
                },
            ],
        },
        FixtureEntry {
            surface: "app-data".to_string(),
            file: "parity/fixtures/app-data.json".to_string(),
            source_refs: vec![
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/appDataHexToCid.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/appDataHexToCid.spec.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/cidToAppDataHex.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/cidToAppDataHex.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/generateAppDataDoc.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/generateAppDataDoc.spec.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/getAppDataInfo.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/getAppDataInfo.spec.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/getAppDataSchema.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/getAppDataSchema.spec.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/validateAppDataDoc.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/validateAppDataDoc.spec.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/fetchDocFromCid.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/fetchDocFromCid.spec.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/fetchDocFromAppData.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/fetchDocFromAppData.spec.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/uploadMetadataDocToIpfsLegacy.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/api/uploadMetadataDocToIpfsLegacy.spec.ts"
                        .to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/types.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/consts.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/importSchema.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/utils/ipfs.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/utils/stringify.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/generatedTypes/index.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/generatedTypes/latest.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/src/mocks.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/test/schema.spec.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/test/schema-1.5.0.spec.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/test/flashloan-v1.7.0.spec.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/app-data/test/wrappers.v1.13.0.spec.ts".to_string(),
                },
            ],
        },
        FixtureEntry {
            surface: "orderbook".to_string(),
            file: "parity/fixtures/orderbook.json".to_string(),
            source_refs: vec![
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/order-book/src/api.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/order-book/src/api.spec.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/order-book/src/request.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/order-book/src/request.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/order-book/src/transformOrder.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/order-book/src/transformOrder.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/order-book/src/types.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "services".to_string(),
                    path: "crates/orderbook/openapi.yml".to_string(),
                },
                FixtureSourceRef {
                    repo: "services".to_string(),
                    path: "crates/shared/src/order_validation.rs".to_string(),
                },
                FixtureSourceRef {
                    repo: "services".to_string(),
                    path: "crates/orderbook/src/app_data.rs".to_string(),
                },
            ],
        },
        FixtureEntry {
            surface: "trading".to_string(),
            file: "parity/fixtures/trading.json".to_string(),
            source_refs: vec![
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/index.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/types.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/appDataUtils.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/appDataUtils.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/getQuote.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/getQuote.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/getOrderToSign.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/getOrderToSign.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/postSwapOrder.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/postSwapOrder.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/postCoWProtocolTrade.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/postCoWProtocolTrade.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/postLimitOrder.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/postLimitOrder.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/postSellNativeCurrencyOrder.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/postSellNativeCurrencyOrder.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/getOrderTypedData.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/getPreSignTransaction.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/getPreSignTransaction.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/getEthFlowTransaction.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/getEthFlowTransaction.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/getSettlementContract.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/getSettlementContract.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/onChainCancellation.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/onChainCancellation.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/resolveSlippageSuggestion.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/resolveSlippageSuggestion.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/suggestSlippageBps.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/suggestSlippageBps.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/suggestSlippageFromFee.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/suggestSlippageFromFee.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/suggestSlippageFromVolume.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/suggestSlippageFromVolume.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/tradingSdk.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/tradingSdk.test.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/trading/src/calculateUniqueOrderId.test.ts".to_string(),
                },
            ],
        },
        FixtureEntry {
            surface: "subgraph".to_string(),
            file: "parity/fixtures/subgraph.json".to_string(),
            source_refs: vec![
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/subgraph/src/api.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/subgraph/src/queries.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/subgraph/src/graphql.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/subgraph/src/api.spec.ts".to_string(),
                },
            ],
        },
        FixtureEntry {
            surface: "sdk".to_string(),
            file: "parity/fixtures/sdk.json".to_string(),
            source_refs: vec![
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/sdk/src/index.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/sdk/src/typedoc-entry.ts".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/sdk/package.json".to_string(),
                },
                FixtureSourceRef {
                    repo: "cow-sdk".to_string(),
                    path: "packages/sdk/README.md".to_string(),
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
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

    fn init_repo(root: &Path, producer_paths: &[&str]) -> Result<String> {
        fs::create_dir_all(root).with_context(|| format!("failed to create {}", root.display()))?;
        run_git(root, &["init"])?;
        run_git(root, &["config", "user.email", "tests@example.com"])?;
        run_git(root, &["config", "user.name", "Parity Maintainer Tests"])?;

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

        run_git(root, &["add", "."])?;
        run_git(root, &["commit", "-m", "initial fixture sources"])?;
        git_stdout(root, &["rev-parse", "HEAD"])
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

    fn write_app_data_schema_fixtures(workspace_root: &Path, cow_sdk_root: &Path) -> Result<()> {
        let source = cow_sdk_root.join(APP_DATA_SCHEMA_SOURCE_DIR);
        let vendored = workspace_root.join(APP_DATA_SCHEMA_VENDOR_DIR);

        for path in [&source, &vendored] {
            fs::create_dir_all(path)
                .with_context(|| format!("failed to create {}", path.display()))?;
        }

        let files = [
            (
                "definitions.json",
                "{\"$id\":\"https://example.invalid/definitions.json\",\"type\":\"object\"}\n",
            ),
            (
                "v1.14.0.json",
                "{\"$id\":\"https://example.invalid/v1.14.0.json\",\"type\":\"object\"}\n",
            ),
            (
                "quote/v1.1.0.json",
                "{\"$id\":\"https://example.invalid/quote/v1.1.0.json\",\"type\":\"object\"}\n",
            ),
        ];

        for (relative, contents) in files {
            for root in [&source, &vendored] {
                let path = root.join(relative);
                let parent = path
                    .parent()
                    .with_context(|| format!("missing parent for {}", path.display()))?;
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
                fs::write(&path, contents)
                    .with_context(|| format!("failed to write {}", path.display()))?;
            }
        }

        Ok(())
    }

    struct TestWorkspace {
        root: PathBuf,
        cow_sdk_root: PathBuf,
        contracts_root: PathBuf,
        services_root: PathBuf,
    }

    impl TestWorkspace {
        fn new(name: &str) -> Result<Self> {
            let root = unique_temp_dir(name);
            fs::create_dir_all(&root)
                .with_context(|| format!("failed to create {}", root.display()))?;

            let cow_sdk_root = root.join("cow-sdk");
            let contracts_root = root.join("contracts");
            let services_root = root.join("services");

            let cow_sdk_commit = init_repo(&cow_sdk_root, COW_SDK_PATHS)?;
            let contracts_commit = init_repo(&contracts_root, CONTRACTS_PATHS)?;
            let services_commit = init_repo(&services_root, SERVICES_PATHS)?;

            let repo_commits = BTreeMap::from([
                ("cow-sdk".to_string(), cow_sdk_commit),
                ("contracts".to_string(), contracts_commit),
                ("services".to_string(), services_commit),
            ]);
            write_fixture_files(&root, &repo_commits)?;
            write_app_data_schema_fixtures(&root, &cow_sdk_root)?;

            Ok(Self {
                root,
                cow_sdk_root,
                contracts_root,
                services_root,
            })
        }

        fn cli_options(&self) -> CliOptions {
            CliOptions {
                source_lock: self.root.join("parity/source-lock.yaml"),
                output: self.root.join("parity/source-lock.yaml"),
                cow_sdk_root: Some(self.cow_sdk_root.clone()),
                contracts_root: Some(self.contracts_root.clone()),
                services_root: Some(self.services_root.clone()),
            }
        }

        fn vendored_schema_dir(&self) -> PathBuf {
            self.root.join(APP_DATA_SCHEMA_VENDOR_DIR)
        }

        fn upstream_schema_dir(&self) -> PathBuf {
            self.cow_sdk_root.join(APP_DATA_SCHEMA_SOURCE_DIR)
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
    fn validate_with_roots_rejects_vendored_schema_drift() -> Result<()> {
        let _lock = cwd_lock().lock().expect("cwd lock poisoned");
        let workspace = TestWorkspace::new("schema-drift")?;
        let _guard = CwdGuard::change_to(&workspace.root)?;
        let options = workspace.cli_options();

        snapshot(&options)?;

        let drifted = workspace.vendored_schema_dir().join("v1.14.0.json");
        fs::write(&drifted, "{\"drifted\":true}\n")
            .with_context(|| format!("failed to write {}", drifted.display()))?;

        let error = validate(&options).expect_err("validate should fail on vendored schema drift");
        assert!(
            format!("{error:#}").contains("vendored cow-rs app-data schema bundle does not match"),
            "unexpected error: {error:#}"
        );

        Ok(())
    }

    #[test]
    fn vendor_app_data_schemas_restores_vendored_tree() -> Result<()> {
        let _lock = cwd_lock().lock().expect("cwd lock poisoned");
        let workspace = TestWorkspace::new("vendor-schemas")?;
        let _guard = CwdGuard::change_to(&workspace.root)?;
        let options = workspace.cli_options();

        snapshot(&options)?;

        let vendored = workspace.vendored_schema_dir();
        fs::write(vendored.join("v1.14.0.json"), "{\"drifted\":true}\n")
            .with_context(|| format!("failed to drift {}", vendored.display()))?;
        let stale = vendored.join("stale.json");
        fs::write(&stale, "{\"stale\":true}\n")
            .with_context(|| format!("failed to write {}", stale.display()))?;

        vendor_app_data_schemas(&options)?;

        let upstream = workspace.upstream_schema_dir();
        ensure_matching_file_trees(
            &upstream,
            &vendored,
            "upstream cow-sdk app-data schema bundle",
            "vendored cow-rs app-data schema bundle",
        )?;

        assert!(
            !stale.exists(),
            "stale vendored file should be removed during vendor sync"
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
        let services_entry = lock
            .repositories
            .iter_mut()
            .find(|repo| repo.id == "services")
            .context("missing services entry in generated source lock")?;
        services_entry.commit = "1111111111111111111111111111111111111111".to_string();
        fs::write(
            &options.source_lock,
            serde_yaml::to_string(&lock).context("failed to serialize mutated source lock")?,
        )
        .with_context(|| format!("failed to write {}", options.source_lock.display()))?;

        let fixture_path = workspace.root.join("parity/fixtures/orderbook.json");
        let mut fixture_json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&fixture_path)
                .with_context(|| format!("failed to read {}", fixture_path.display()))?,
        )
        .context("failed to parse orderbook fixture json")?;
        let source_refs = fixture_json["source_refs"]
            .as_array_mut()
            .context("missing orderbook source_refs array")?;
        for source_ref in source_refs {
            if source_ref["repo"].as_str() == Some("services") {
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
            cow_sdk_root: None,
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

        let fixture_path = workspace.root.join("parity/fixtures/sdk.json");
        let mut fixture_json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&fixture_path)
                .with_context(|| format!("failed to read {}", fixture_path.display()))?,
        )
        .context("failed to parse fixture json")?;
        fixture_json["source_refs"][0]["commit"] =
            serde_json::Value::String("2222222222222222222222222222222222222222".to_string());
        fs::write(
            &fixture_path,
            serde_json::to_string_pretty(&fixture_json).context("failed to serialize fixture")?,
        )
        .with_context(|| format!("failed to write {}", fixture_path.display()))?;

        let standalone = CliOptions {
            source_lock: options.source_lock.clone(),
            output: options.output.clone(),
            cow_sdk_root: None,
            contracts_root: None,
            services_root: None,
        };

        let error =
            validate(&standalone).expect_err("validate should fail on fixture commit drift");
        assert!(
            format!("{error:#}").contains("embeds stale commit for repo cow-sdk"),
            "unexpected error: {error:#}"
        );

        Ok(())
    }
}
