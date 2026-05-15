//! Compile-time validator for deployment registry and coverage manifests.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Deserialize;
use sha2::{Digest, Sha256};

include!("src/chain_ids.rs");

const SCHEMA_VERSION: u32 = 2;
const MANIFEST_PATH: &str = "registry.toml";
const PROVENANCE_PATH: &str = "deployment-provenance.yaml";
const COVERAGE_PATH: &str = "deployment-coverage.yaml";

fn main() {
    println!("cargo:rerun-if-changed={MANIFEST_PATH}");
    println!("cargo:rerun-if-changed={PROVENANCE_PATH}");
    println!("cargo:rerun-if-changed={COVERAGE_PATH}");
    println!("cargo:rerun-if-changed=src/chain_ids.rs");
    println!("cargo:rerun-if-changed=abi/cow-shed/proxy-creation-code");

    let manifest = read_toml_manifest();
    let provenance = read_yaml_manifest::<ProvenanceManifest>(PROVENANCE_PATH);
    let coverage = read_yaml_manifest::<CoverageManifest>(COVERAGE_PATH);
    let supported: BTreeSet<u64> = DEPLOYMENT_CHAIN_IDS.iter().copied().collect();

    if manifest.schema_version != SCHEMA_VERSION {
        fail(&format!(
            "{MANIFEST_PATH}: unsupported schema_version {}; expected {SCHEMA_VERSION}",
            manifest.schema_version
        ));
    }
    if provenance.version != SCHEMA_VERSION {
        fail(&format!(
            "{PROVENANCE_PATH}: unsupported version {}; expected {SCHEMA_VERSION}",
            provenance.version
        ));
    }
    if coverage.schema_version != SCHEMA_VERSION {
        fail(&format!(
            "{COVERAGE_PATH}: unsupported schema_version {}; expected {SCHEMA_VERSION}",
            coverage.schema_version
        ));
    }

    let registry_entries = validate_registry_manifest(&manifest.entries, &supported);
    let provenance_entries = validate_provenance_manifest(&provenance.deployments, &supported);
    validate_registry_provenance_lockstep(&registry_entries, &provenance_entries);
    validate_coverage_manifest(&coverage.coverage, &registry_entries, &supported);
    validate_cow_shed_proxy_artifacts();
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
struct ProvenanceManifest {
    version: u32,
    #[allow(dead_code, reason = "metadata shape is validated by serde")]
    generated_at_utc: Option<String>,
    #[serde(default)]
    deployments: Vec<ProvenanceEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProvenanceEntry {
    contract_id: String,
    chain_id: u64,
    env: String,
    address: String,
    source_repo: String,
    source_commit: String,
    source_path: String,
    source_symbol: String,
    verification: VerificationEntry,
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

fn validate_provenance_manifest(
    entries: &[ProvenanceEntry],
    supported: &BTreeSet<u64>,
) -> BTreeMap<(String, u64, String), RegistryEvidence> {
    let mut seen = BTreeMap::new();

    for (index, entry) in entries.iter().enumerate() {
        let row = index + 1;
        validate_contract_id(PROVENANCE_PATH, row, &entry.contract_id);
        validate_env_scope(PROVENANCE_PATH, row, &entry.contract_id, &entry.env);
        validate_chain_id(PROVENANCE_PATH, row, entry.chain_id, supported);
        validate_address(PROVENANCE_PATH, row, &entry.address);
        validate_verification(PROVENANCE_PATH, row, &entry.verification);
        for (field, value) in [
            ("source_repo", entry.source_repo.as_str()),
            ("source_path", entry.source_path.as_str()),
            ("source_symbol", entry.source_symbol.as_str()),
        ] {
            if value.trim().is_empty() {
                fail(&format!(
                    "{PROVENANCE_PATH}: entry #{row} has empty {field}"
                ));
            }
        }
        if !is_40_byte_hex_without_prefix(&entry.source_commit) {
            fail(&format!(
                "{PROVENANCE_PATH}: entry #{row} has malformed source_commit `{}`",
                entry.source_commit
            ));
        }

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
                "{PROVENANCE_PATH}: entry #{row} duplicates (contract_id=`{}`, chain_id={}, env=`{}`)",
                entry.contract_id, entry.chain_id, entry.env
            ));
        }
    }

    seen
}

fn validate_registry_provenance_lockstep(
    registry: &BTreeMap<(String, u64, String), RegistryEvidence>,
    provenance: &BTreeMap<(String, u64, String), RegistryEvidence>,
) {
    for (key, registry_row) in registry {
        let Some(provenance_row) = provenance.get(key) else {
            fail(&format!(
                "{PROVENANCE_PATH}: missing provenance row for (contract_id=`{}`, chain_id={}, env=`{}`)",
                key.0, key.1, key.2
            ));
        };
        if registry_row.address != provenance_row.address {
            fail(&format!(
                "{PROVENANCE_PATH}: address mismatch for (contract_id=`{}`, chain_id={}, env=`{}`)",
                key.0, key.1, key.2
            ));
        }
        if registry_row.verification_status != provenance_row.verification_status {
            fail(&format!(
                "{PROVENANCE_PATH}: verification mismatch for (contract_id=`{}`, chain_id={}, env=`{}`)",
                key.0, key.1, key.2
            ));
        }
    }

    for key in provenance.keys() {
        if !registry.contains_key(key) {
            fail(&format!(
                "{PROVENANCE_PATH}: provenance row for (contract_id=`{}`, chain_id={}, env=`{}`) has no matching registry row",
                key.0, key.1, key.2
            ));
        }
    }
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

fn is_40_byte_hex_without_prefix(candidate: &str) -> bool {
    candidate.len() == 40 && candidate.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn validate_cow_shed_proxy_artifacts() {
    for version in ["1.0.0", "1.0.1"] {
        let code_path = format!("abi/cow-shed/proxy-creation-code/v{version}.bin");
        let digest_path = format!("{code_path}.sha256");
        let bytes = std::fs::read(&code_path)
            .unwrap_or_else(|source| fail(&format!("failed to read `{code_path}`: {source}")));
        if bytes.is_empty() {
            fail(&format!("{code_path}: proxy init code must not be empty"));
        }
        let expected = read_to_string(&digest_path);
        let actual = hex_lower(Sha256::digest(&bytes).as_slice());
        if expected.trim() != actual {
            fail(&format!(
                "{digest_path}: SHA-256 mismatch for COW Shed {version} proxy init code"
            ));
        }
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX[(byte >> 4) as usize]));
        out.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    out
}

fn fail(message: &str) -> ! {
    eprintln!("error: {message}");
    std::process::exit(1);
}
