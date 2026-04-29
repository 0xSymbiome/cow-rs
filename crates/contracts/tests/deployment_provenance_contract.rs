use std::collections::BTreeMap;
use std::fmt;

use cow_sdk_contracts::{ContractId, Registry};
use cow_sdk_core::{Address, CowEnv, SupportedChainId};
use serde::Deserialize;

const HAPPY_PROVENANCE: &str = include_str!("fixtures/deployment-provenance-happy.yaml");
const MISSING_ROW_PROVENANCE: &str =
    include_str!("fixtures/deployment-provenance-missing-row.yaml");
const SKIPPED_PROVENANCE: &str = include_str!("fixtures/deployment-provenance-skipped.yaml");

#[test]
fn every_registry_row_has_provenance() {
    validate_registry_rows_have_provenance(HAPPY_PROVENANCE)
        .expect("happy-path provenance fixture must cover every registry row");
}

#[test]
fn every_provenance_chain_id_is_supported() {
    let entries = provenance_entries_by_key(HAPPY_PROVENANCE)
        .expect("happy-path provenance fixture must parse");

    for key in entries.keys() {
        SupportedChainId::try_from(key.chain_id)
            .unwrap_or_else(|_| panic!("provenance entry references unsupported {key}"));
    }
}

#[test]
fn live_confirmation_kind_is_code_hash() {
    validate_release_live_confirmation(HAPPY_PROVENANCE)
        .expect("happy-path release fixture must use code_hash confirmations");
}

#[test]
fn missing_row_fixture_is_rejected() {
    let error = validate_registry_rows_have_provenance(MISSING_ROW_PROVENANCE)
        .expect_err("fixture with a dropped registry row must be rejected");

    assert!(
        error.contains("missing provenance row for Settlement / chain 1 / prod"),
        "missing-row fixture must fail on the dropped registry key, got: {error}",
    );
}

#[test]
fn release_invalid_skipped_fixture_is_rejected() {
    let error = validate_release_live_confirmation(SKIPPED_PROVENANCE)
        .expect_err("release-mode validation must reject skipped confirmations");

    assert!(
        error.contains("RELEASE-INVALID") && error.contains("live_confirmation.kind `skipped`"),
        "skipped fixture must be classified as release-invalid, got: {error}",
    );
}

fn validate_registry_rows_have_provenance(source: &str) -> Result<(), String> {
    let registry = Registry::default();
    let provenance = provenance_entries_by_key(source)?;

    for (contract_id, chain_id, env, address) in registry.entries() {
        let key = ProvenanceKey::new(contract_id, u64::from(chain_id), env);
        let entry = provenance
            .get(&key)
            .ok_or_else(|| format!("missing provenance row for {key}"))?;

        if entry.address != *address {
            return Err(format!(
                "address mismatch for {key}: registry has {}, provenance has {}",
                address.as_str(),
                entry.address.as_str(),
            ));
        }
    }

    Ok(())
}

fn validate_release_live_confirmation(source: &str) -> Result<(), String> {
    let entries = provenance_entries_by_key(source)?;

    for (key, entry) in entries {
        if entry.live_confirmation.kind != "code_hash" {
            return Err(format!(
                "RELEASE-INVALID: {key} has live_confirmation.kind `{}`",
                entry.live_confirmation.kind,
            ));
        }

        let Some(code_hash) = entry.live_confirmation.code_hash.as_deref() else {
            return Err(format!("RELEASE-INVALID: {key} has no code_hash"));
        };
        if !is_32_byte_hex(code_hash) {
            return Err(format!(
                "RELEASE-INVALID: {key} has malformed code_hash `{code_hash}`",
            ));
        }
    }

    Ok(())
}

fn provenance_entries_by_key(
    source: &str,
) -> Result<BTreeMap<ProvenanceKey, ProvenanceEntry>, String> {
    let manifest: ProvenanceManifest = serde_yaml::from_str(source)
        .map_err(|error| format!("failed to parse provenance fixture: {error}"))?;
    if manifest.version != 1 {
        return Err(format!(
            "unsupported provenance fixture version {}; expected 1",
            manifest.version,
        ));
    }

    let mut entries = BTreeMap::new();
    for entry in manifest.provenance {
        let chain_id = SupportedChainId::try_from(entry.chain_id)
            .map_err(|_| format!("unsupported provenance chain id {}", entry.chain_id))?;
        if entry.live_confirmation.rpc_chain_id != entry.chain_id {
            return Err(format!(
                "{} / chain {} / {} has rpc_chain_id {}",
                entry.contract_id,
                entry.chain_id,
                entry.env.as_str(),
                entry.live_confirmation.rpc_chain_id,
            ));
        }

        let key = ProvenanceKey::new(entry.contract_id, u64::from(chain_id), entry.env);
        if entries.insert(key.clone(), entry).is_some() {
            return Err(format!("duplicate provenance row for {key}"));
        }
    }

    Ok(entries)
}

fn is_32_byte_hex(value: &str) -> bool {
    let Some(body) = value.strip_prefix("0x") else {
        return false;
    };
    body.len() == 64 && body.chars().all(|ch| ch.is_ascii_hexdigit())
}

#[derive(Debug, Deserialize)]
struct ProvenanceManifest {
    version: u32,
    #[serde(default)]
    provenance: Vec<ProvenanceEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ProvenanceEntry {
    contract_id: ContractId,
    chain_id: u64,
    env: CowEnv,
    address: Address,
    live_confirmation: LiveConfirmation,
}

#[derive(Debug, Clone, Deserialize)]
struct LiveConfirmation {
    kind: String,
    code_hash: Option<String>,
    rpc_chain_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProvenanceKey {
    contract_id: ContractId,
    chain_id: u64,
    env: &'static str,
}

impl ProvenanceKey {
    const fn new(contract_id: ContractId, chain_id: u64, env: CowEnv) -> Self {
        Self {
            contract_id,
            chain_id,
            env: env.as_str(),
        }
    }
}

impl fmt::Display for ProvenanceKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} / chain {} / {}",
            self.contract_id, self.chain_id, self.env,
        )
    }
}
