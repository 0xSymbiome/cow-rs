//! Compile-time validator for the chain-keyed deployment registry manifest.
//!
//! Parses `crates/contracts/registry.toml` through the same TOML dialect the
//! runtime loader uses and rejects any row that violates the reviewed
//! invariants with a precise diagnostic that names the offending manifest
//! line so operators see an actionable fix target rather than a generic
//! parse error.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Deserialize;

// The authoritative chain-id set is declared in a single shared include
// that the `src/` tree consumes as a Rust module. The build script cannot
// depend on the same crate's compiled output without inviting a circular
// build, so the include! brings the same literal into the build context.
include!("src/chain_ids.rs");

const SCHEMA_VERSION: u32 = 1;
const MANIFEST_PATH: &str = "registry.toml";
const PROVENANCE_PATH: &str = "deployment-provenance.yaml";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestSchema {
    schema_version: u32,
    #[serde(default)]
    entries: Vec<ManifestEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestEntry {
    contract_id: String,
    chain_id: u64,
    env: String,
    address: String,
}

fn main() {
    println!("cargo:rerun-if-changed=registry.toml");
    println!("cargo:rerun-if-changed=deployment-provenance.yaml");
    println!("cargo:rerun-if-changed=src/chain_ids.rs");

    let manifest = read_registry_manifest();
    let provenance = read_provenance_manifest();

    if manifest.schema_version != SCHEMA_VERSION {
        let actual = manifest.schema_version;
        fail(&format!(
            "{MANIFEST_PATH}: unsupported schema_version {actual}; expected {SCHEMA_VERSION}",
        ));
    }

    let supported: BTreeSet<u64> = SUPPORTED_CHAIN_IDS.iter().copied().collect();
    let registry_entries = validate_registry_manifest(&manifest.entries, &supported);
    let provenance_entries = validate_provenance_manifest(&provenance, &supported);

    for (key, registry_address) in &registry_entries {
        let Some(provenance_address) = provenance_entries.get(key) else {
            fail(&format!(
                "{PROVENANCE_PATH}: missing provenance row for (contract_id=`{}`, chain_id={}, env=`{}`)",
                key.0, key.1, key.2,
            ));
        };

        if registry_address != provenance_address {
            fail(&format!(
                "{PROVENANCE_PATH}: address mismatch for (contract_id=`{}`, chain_id={}, env=`{}`): registry has `{registry_address}`, provenance has `{provenance_address}`",
                key.0, key.1, key.2,
            ));
        }
    }

    for key in provenance_entries.keys() {
        if !registry_entries.contains_key(key) {
            fail(&format!(
                "{PROVENANCE_PATH}: provenance row for (contract_id=`{}`, chain_id={}, env=`{}`) has no matching registry row",
                key.0, key.1, key.2,
            ));
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProvenanceManifestSchema {
    version: u32,
    #[allow(
        dead_code,
        reason = "shape validation accepts and preserves the reviewed metadata field"
    )]
    generated_at_utc: Option<String>,
    #[serde(default)]
    provenance: Vec<ProvenanceEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProvenanceEntry {
    contract_id: String,
    chain_id: u64,
    env: String,
    address: String,
    authority: String,
    source_repo: String,
    source_commit: String,
    source_path: String,
    #[allow(
        dead_code,
        reason = "source_key is optional for non-JSON source authority rows"
    )]
    source_key: Option<String>,
    source_symbol: String,
    live_confirmation: LiveConfirmation,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveConfirmation {
    kind: String,
    code_hash: Option<String>,
    #[allow(dead_code, reason = "selector probes are optional release evidence")]
    selector_check: Option<SelectorCheck>,
    rpc_chain_id: u64,
    #[allow(
        dead_code,
        reason = "release validation, not build.rs, owns timestamp freshness"
    )]
    confirmed_at: String,
    #[allow(
        dead_code,
        reason = "release validation, not build.rs, owns confirmer identity"
    )]
    confirmer: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectorCheck {
    #[allow(
        dead_code,
        reason = "build.rs validates the selector_check object shape only"
    )]
    enabled: bool,
    #[allow(
        dead_code,
        reason = "selector is populated only when selector probes are enabled"
    )]
    selector: Option<String>,
    #[allow(
        dead_code,
        reason = "result is populated only when selector probes are enabled"
    )]
    result: Option<String>,
    #[allow(
        dead_code,
        reason = "error is populated only when selector probes fail"
    )]
    error: Option<String>,
}

fn read_registry_manifest() -> ManifestSchema {
    let raw = read_to_string(MANIFEST_PATH);
    match toml::from_str(&raw) {
        Ok(parsed) => parsed,
        Err(error) => {
            fail(&format!("{MANIFEST_PATH}: malformed TOML — {error}"));
        }
    }
}

fn read_provenance_manifest() -> ProvenanceManifestSchema {
    let raw = read_to_string(PROVENANCE_PATH);
    match serde_yaml::from_str(&raw) {
        Ok(parsed) => parsed,
        Err(error) => {
            fail(&format!("{PROVENANCE_PATH}: malformed YAML — {error}"));
        }
    }
}

fn read_to_string(path: &str) -> String {
    let path = Path::new(path);
    std::fs::read_to_string(path).unwrap_or_else(|source| {
        let display = path.display();
        fail(&format!("failed to read `{display}`: {source}"));
    })
}

fn validate_registry_manifest(
    entries: &[ManifestEntry],
    supported: &BTreeSet<u64>,
) -> BTreeMap<(String, u64, String), String> {
    let mut seen: BTreeMap<(String, u64, String), String> = BTreeMap::new();

    for (index, entry) in entries.iter().enumerate() {
        let row = index + 1;

        validate_contract_id(MANIFEST_PATH, row, &entry.contract_id);
        validate_env(MANIFEST_PATH, row, &entry.env);
        validate_chain_id(MANIFEST_PATH, row, entry.chain_id, supported);
        if !is_valid_ethereum_address(&entry.address) {
            let address = &entry.address;
            fail(&format!(
                "{MANIFEST_PATH}: entry #{row} declares malformed address `{address}` — expected a 0x-prefixed 40-character hex literal",
            ));
        }

        let key = (entry.contract_id.clone(), entry.chain_id, entry.env.clone());
        if seen.insert(key, entry.address.clone()).is_some() {
            let contract_id = &entry.contract_id;
            let chain_id = entry.chain_id;
            let env = &entry.env;
            fail(&format!(
                "{MANIFEST_PATH}: entry #{row} duplicates (contract_id=`{contract_id}`, chain_id={chain_id}, env=`{env}`)",
            ));
        }
    }

    seen
}

fn validate_provenance_manifest(
    manifest: &ProvenanceManifestSchema,
    supported: &BTreeSet<u64>,
) -> BTreeMap<(String, u64, String), String> {
    if manifest.version != SCHEMA_VERSION {
        let actual = manifest.version;
        fail(&format!(
            "{PROVENANCE_PATH}: unsupported version {actual}; expected {SCHEMA_VERSION}",
        ));
    }

    let mut seen: BTreeMap<(String, u64, String), String> = BTreeMap::new();

    for (index, entry) in manifest.provenance.iter().enumerate() {
        let row = index + 1;
        validate_contract_id(PROVENANCE_PATH, row, &entry.contract_id);
        validate_env(PROVENANCE_PATH, row, &entry.env);
        validate_chain_id(PROVENANCE_PATH, row, entry.chain_id, supported);
        validate_provenance_metadata(row, entry);
        validate_live_confirmation(row, entry);

        if !is_valid_ethereum_address(&entry.address) {
            let address = &entry.address;
            fail(&format!(
                "{PROVENANCE_PATH}: entry #{row} declares malformed address `{address}` — expected a 0x-prefixed 40-character hex literal",
            ));
        }

        let key = (entry.contract_id.clone(), entry.chain_id, entry.env.clone());
        if seen.insert(key, entry.address.clone()).is_some() {
            let contract_id = &entry.contract_id;
            let chain_id = entry.chain_id;
            let env = &entry.env;
            fail(&format!(
                "{PROVENANCE_PATH}: entry #{row} duplicates (contract_id=`{contract_id}`, chain_id={chain_id}, env=`{env}`)",
            ));
        }
    }

    seen
}

fn validate_contract_id(path: &str, row: usize, contract_id: &str) {
    match contract_id {
        "Settlement" | "VaultRelayer" | "EthFlow" => {}
        other => fail(&format!(
            "{path}: entry #{row} declares unknown contract_id `{other}`; expected one of Settlement, VaultRelayer, EthFlow",
        )),
    }
}

fn validate_env(path: &str, row: usize, env: &str) {
    match env {
        "prod" | "staging" => {}
        other => fail(&format!(
            "{path}: entry #{row} declares unknown env `{other}`; expected prod or staging",
        )),
    }
}

fn validate_chain_id(path: &str, row: usize, chain_id: u64, supported: &BTreeSet<u64>) {
    if !supported.contains(&chain_id) {
        fail(&format!(
            "{path}: entry #{row} declares unsupported chain_id {chain_id}; expected one of the eleven supported chain ids",
        ));
    }
}

fn validate_provenance_metadata(row: usize, entry: &ProvenanceEntry) {
    match entry.authority.as_str() {
        "primary" | "secondary" | "release-smoke" => {}
        other => fail(&format!(
            "{PROVENANCE_PATH}: entry #{row} declares unknown authority `{other}`; expected primary, secondary, or release-smoke",
        )),
    }

    for (field, value) in [
        ("source_repo", entry.source_repo.as_str()),
        ("source_commit", entry.source_commit.as_str()),
        ("source_path", entry.source_path.as_str()),
        ("source_symbol", entry.source_symbol.as_str()),
    ] {
        if value.trim().is_empty() {
            fail(&format!(
                "{PROVENANCE_PATH}: entry #{row} has empty {field}",
            ));
        }
    }

    if !is_40_byte_hex_without_prefix(&entry.source_commit) {
        let source_commit = &entry.source_commit;
        fail(&format!(
            "{PROVENANCE_PATH}: entry #{row} has malformed source_commit `{source_commit}` — expected a 40-character git commit hex",
        ));
    }
}

fn validate_live_confirmation(row: usize, entry: &ProvenanceEntry) {
    let confirmation = &entry.live_confirmation;
    if confirmation.kind != "code_hash" {
        let kind = &confirmation.kind;
        fail(&format!(
            "{PROVENANCE_PATH}: entry #{row} declares live_confirmation.kind `{kind}`; release-facing provenance must use code_hash",
        ));
    }

    let Some(code_hash) = confirmation.code_hash.as_deref() else {
        fail(&format!(
            "{PROVENANCE_PATH}: entry #{row} is missing live_confirmation.code_hash",
        ));
    };
    if !is_32_byte_hex(code_hash) {
        fail(&format!(
            "{PROVENANCE_PATH}: entry #{row} has malformed live_confirmation.code_hash `{code_hash}`",
        ));
    }

    if confirmation.rpc_chain_id != entry.chain_id {
        let rpc_chain_id = confirmation.rpc_chain_id;
        let chain_id = entry.chain_id;
        fail(&format!(
            "{PROVENANCE_PATH}: entry #{row} has live_confirmation.rpc_chain_id {rpc_chain_id}, expected {chain_id}",
        ));
    }
}

fn is_valid_ethereum_address(candidate: &str) -> bool {
    let Some(body) = candidate.strip_prefix("0x") else {
        return false;
    };
    body.len() == 40 && body.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_32_byte_hex(candidate: &str) -> bool {
    let Some(body) = candidate.strip_prefix("0x") else {
        return false;
    };
    body.len() == 64 && body.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_40_byte_hex_without_prefix(candidate: &str) -> bool {
    candidate.len() == 40 && candidate.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn fail(message: &str) -> ! {
    // Emit a cargo-visible error line so the offending manifest detail is
    // surfaced through every cargo invocation, then halt the build so a
    // malformed registry cannot land silently.
    eprintln!("error: {message}");
    std::process::exit(1);
}
