//! Compile-time validator for deployment registry and coverage manifests.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Deserialize;

include!("src/chain_ids.rs");

const SCHEMA_VERSION: u32 = 2;
const MANIFEST_PATH: &str = "registry.toml";
const COVERAGE_PATH: &str = "deployment-coverage.yaml";

fn main() {
    println!("cargo:rerun-if-changed={MANIFEST_PATH}");
    println!("cargo:rerun-if-changed={COVERAGE_PATH}");
    println!("cargo:rerun-if-changed=src/chain_ids.rs");

    let manifest = read_toml_manifest();
    let coverage = read_yaml_manifest::<CoverageManifest>(COVERAGE_PATH);
    let supported: BTreeSet<u64> = DEPLOYMENT_CHAIN_IDS.iter().copied().collect();

    if manifest.schema_version != SCHEMA_VERSION {
        fail(&format!(
            "{MANIFEST_PATH}: unsupported schema_version {}; expected {SCHEMA_VERSION}",
            manifest.schema_version
        ));
    }
    if coverage.schema_version != SCHEMA_VERSION {
        fail(&format!(
            "{COVERAGE_PATH}: unsupported schema_version {}; expected {SCHEMA_VERSION}",
            coverage.schema_version
        ));
    }

    let registry_entries = validate_registry_manifest(&manifest.entries, &supported);
    validate_coverage_manifest(&coverage.coverage, &registry_entries, &supported);
}

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
    verification: VerificationEntry,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VerificationEntry {
    status: String,
    source: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageManifest {
    schema_version: u32,
    #[serde(default)]
    coverage: Vec<CoverageEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoverageEntry {
    contract_id: String,
    chain_id: u64,
    status: String,
    evidence: String,
}

fn read_toml_manifest() -> ManifestSchema {
    let raw = read_to_string(MANIFEST_PATH);
    toml::from_str(&raw).unwrap_or_else(|error| fail(&format!("{MANIFEST_PATH}: {error}")))
}

fn read_yaml_manifest<T>(path: &str) -> T
where
    T: for<'de> Deserialize<'de>,
{
    let raw = read_to_string(path);
    let mut documents = serde_yaml::Deserializer::from_str(&raw);
    let Some(document) = documents.next() else {
        fail(&format!("{path}: empty YAML document"));
    };
    T::deserialize(document).unwrap_or_else(|error| fail(&format!("{path}: {error}")))
}

fn read_to_string(path: &str) -> String {
    std::fs::read_to_string(Path::new(path))
        .unwrap_or_else(|source| fail(&format!("failed to read `{path}`: {source}")))
}

fn validate_registry_manifest(
    entries: &[ManifestEntry],
    supported: &BTreeSet<u64>,
) -> BTreeMap<(String, u64, String), RegistryEvidence> {
    let mut seen = BTreeMap::new();

    for (index, entry) in entries.iter().enumerate() {
        let row = index + 1;
        validate_contract_id(MANIFEST_PATH, row, &entry.contract_id);
        validate_env_scope(MANIFEST_PATH, row, &entry.contract_id, &entry.env);
        validate_chain_id(MANIFEST_PATH, row, entry.chain_id, supported);
        validate_address(MANIFEST_PATH, row, &entry.address);
        validate_verification(MANIFEST_PATH, row, &entry.verification);

        let key = (entry.contract_id.clone(), entry.chain_id, entry.env.clone());
        if seen
            .insert(
                key,
                RegistryEvidence {
                    address: entry.address.clone(),
                    verification_status: entry.verification.status.clone(),
                },
            )
            .is_some()
        {
            fail(&format!(
                "{MANIFEST_PATH}: entry #{row} duplicates (contract_id=`{}`, chain_id={}, env=`{}`)",
                entry.contract_id, entry.chain_id, entry.env
            ));
        }
    }

    seen
}

fn validate_coverage_manifest(
    entries: &[CoverageEntry],
    registry: &BTreeMap<(String, u64, String), RegistryEvidence>,
    supported: &BTreeSet<u64>,
) {
    let mut seen = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let row = index + 1;
        validate_contract_id(COVERAGE_PATH, row, &entry.contract_id);
        match entry.status.as_str() {
            "not_deployed" | "not_supported" | "out_of_scope" => {}
            other => fail(&format!(
                "{COVERAGE_PATH}: entry #{row} declares unknown status `{other}`"
            )),
        }
        if !supported.contains(&entry.chain_id) && entry.status != "not_supported" {
            fail(&format!(
                "{COVERAGE_PATH}: entry #{row} declares unsupported chain_id {} without not_supported status",
                entry.chain_id
            ));
        }
        if supported.contains(&entry.chain_id) && entry.status == "not_supported" {
            fail(&format!(
                "{COVERAGE_PATH}: entry #{row} marks addressable chain_id {} as not_supported",
                entry.chain_id
            ));
        }
        if entry.evidence.trim().is_empty() {
            fail(&format!("{COVERAGE_PATH}: entry #{row} has empty evidence"));
        }

        let key = (entry.contract_id.clone(), entry.chain_id);
        if !seen.insert(key.clone()) {
            fail(&format!(
                "{COVERAGE_PATH}: entry #{row} duplicates (contract_id=`{}`, chain_id={})",
                entry.contract_id, entry.chain_id
            ));
        }
        if registry
            .keys()
            .any(|registry_key| registry_key.0 == key.0 && registry_key.1 == key.1)
        {
            fail(&format!(
                "{COVERAGE_PATH}: entry #{row} overlaps a registry deployment row"
            ));
        }
    }
}

fn validate_contract_id(path: &str, row: usize, contract_id: &str) {
    match contract_id {
        "Settlement"
        | "VaultRelayer"
        | "EthFlow"
        | "ComposableCow"
        | "ExtensibleFallbackHandler"
        | "CurrentBlockTimestampFactory"
        | "TwapHandler"
        | "GoodAfterTimeHandler"
        | "StopLossHandler"
        | "TradeAboveThresholdHandler"
        | "PerpetualStableSwapHandler"
        | "CowShedImplementation"
        | "CowShedFactory"
        | "CowShedForComposableCow" => {}
        other => fail(&format!(
            "{path}: entry #{row} declares unknown contract_id `{other}`"
        )),
    }
}

fn validate_env_scope(path: &str, row: usize, contract_id: &str, env: &str) {
    let environment_agnostic = matches!(
        contract_id,
        "ComposableCow"
            | "ExtensibleFallbackHandler"
            | "CurrentBlockTimestampFactory"
            | "TwapHandler"
            | "GoodAfterTimeHandler"
            | "StopLossHandler"
            | "TradeAboveThresholdHandler"
            | "PerpetualStableSwapHandler"
            | "CowShedImplementation"
            | "CowShedFactory"
            | "CowShedForComposableCow"
    );
    let valid = if environment_agnostic {
        env == "environment_agnostic"
    } else {
        matches!(env, "prod" | "staging")
    };
    if !valid {
        fail(&format!(
            "{path}: entry #{row} declares invalid env `{env}` for `{contract_id}`"
        ));
    }
}

fn validate_chain_id(path: &str, row: usize, chain_id: u64, supported: &BTreeSet<u64>) {
    if !supported.contains(&chain_id) {
        fail(&format!(
            "{path}: entry #{row} declares unsupported chain_id {chain_id}"
        ));
    }
}

fn validate_address(path: &str, row: usize, address: &str) {
    if !is_valid_ethereum_address(address) {
        fail(&format!(
            "{path}: entry #{row} declares malformed address `{address}`"
        ));
    }
}

fn validate_verification(path: &str, row: usize, verification: &VerificationEntry) {
    match verification.status.as_str() {
        "code_hash_verified"
        | "external_verified"
        | "readme_table_unverified"
        | "canonical_unverified" => {}
        other => fail(&format!(
            "{path}: entry #{row} declares unknown verification.status `{other}`"
        )),
    }
    if verification.source.trim().is_empty() {
        fail(&format!(
            "{path}: entry #{row} has empty verification.source"
        ));
    }
}

#[derive(Debug)]
struct RegistryEvidence {
    address: String,
    verification_status: String,
}

fn is_valid_ethereum_address(candidate: &str) -> bool {
    let Some(body) = candidate.strip_prefix("0x") else {
        return false;
    };
    body.len() == 40 && body.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn fail(message: &str) -> ! {
    eprintln!("error: {message}");
    std::process::exit(1);
}
