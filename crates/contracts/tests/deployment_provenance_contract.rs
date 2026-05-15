use std::collections::BTreeMap;
use std::fmt;

use cow_sdk_contracts::{
    ContractId, DeploymentChainId, DeploymentEnv, DeploymentVerificationStatus, Registry,
};
use cow_sdk_core::Address;
use serde::Deserialize;

const RELEASE_PROVENANCE: &str = include_str!("../deployment-provenance.yaml");

#[test]
fn every_registry_row_has_provenance() {
    let registry = Registry::default();
    let provenance = provenance_entries_by_key(RELEASE_PROVENANCE)
        .expect("release provenance manifest must parse");

    for (contract_id, chain_id, env, address) in registry.entries() {
        let key = ProvenanceKey {
            contract_id,
            chain_id,
            env,
        };
        let entry = provenance
            .get(&key)
            .unwrap_or_else(|| panic!("missing provenance row for {key}"));
        assert_eq!(
            entry.address, *address,
            "provenance address must match registry for {key}",
        );
        assert_eq!(
            registry.verification(contract_id, chain_id, env),
            Some(entry.verification.status),
            "provenance verification must match registry for {key}",
        );
    }
}

#[test]
fn every_provenance_chain_id_is_in_deployment_taxonomy() {
    let entries = provenance_entries_by_key(RELEASE_PROVENANCE)
        .expect("release provenance manifest must parse");

    for key in entries.keys() {
        assert!(
            DeploymentChainId::ALL.contains(&key.chain_id),
            "{key} references a chain outside deployment taxonomy",
        );
    }
}

#[test]
fn source_provenance_fields_are_complete() {
    let entries = provenance_entries_by_key(RELEASE_PROVENANCE)
        .expect("release provenance manifest must parse");

    for (key, entry) in entries {
        assert!(
            !entry.source_repo.trim().is_empty(),
            "{key} must carry source_repo",
        );
        assert!(
            is_40_byte_hex_without_prefix(&entry.source_commit),
            "{key} must carry a pinned 40-character source commit",
        );
        assert!(
            !entry.source_path.trim().is_empty(),
            "{key} must carry source_path",
        );
        assert!(
            !entry.source_symbol.trim().is_empty(),
            "{key} must carry source_symbol",
        );
        assert!(
            !entry.verification.source.trim().is_empty(),
            "{key} must carry verification.source",
        );
    }
}

#[test]
fn provenance_uses_only_registry_verification_taxonomy() {
    let entries = provenance_entries_by_key(RELEASE_PROVENANCE)
        .expect("release provenance manifest must parse");
    let statuses = entries
        .values()
        .map(|entry| entry.verification.status)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(statuses.contains(&DeploymentVerificationStatus::CodeHashVerified));
    assert!(statuses.contains(&DeploymentVerificationStatus::ExternalVerified));
    assert!(statuses.contains(&DeploymentVerificationStatus::ReadmeTableUnverified));
    assert!(statuses.contains(&DeploymentVerificationStatus::CanonicalUnverified));
}

fn provenance_entries_by_key(
    source: &str,
) -> Result<BTreeMap<ProvenanceKey, ProvenanceEntry>, String> {
    let manifest: ProvenanceManifest = serde_json::from_str(source)
        .map_err(|error| format!("failed to parse provenance manifest: {error}"))?;
    if manifest.version != 2 {
        return Err(format!(
            "unsupported provenance version {}; expected 2",
            manifest.version,
        ));
    }

    let mut entries = BTreeMap::new();
    for entry in manifest.deployments {
        let key = ProvenanceKey {
            contract_id: entry.contract_id,
            chain_id: entry.chain_id,
            env: entry.env,
        };
        if entries.insert(key.clone(), entry).is_some() {
            return Err(format!("duplicate provenance row for {key}"));
        }
    }

    Ok(entries)
}

fn is_40_byte_hex_without_prefix(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

#[derive(Debug, Deserialize)]
struct ProvenanceManifest {
    version: u32,
    #[serde(default)]
    deployments: Vec<ProvenanceEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ProvenanceEntry {
    contract_id: ContractId,
    chain_id: DeploymentChainId,
    env: DeploymentEnv,
    address: Address,
    source_repo: String,
    source_commit: String,
    source_path: String,
    source_symbol: String,
    verification: VerificationEntry,
}

#[derive(Debug, Clone, Deserialize)]
struct VerificationEntry {
    status: DeploymentVerificationStatus,
    source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ProvenanceKey {
    contract_id: ContractId,
    chain_id: DeploymentChainId,
    env: DeploymentEnv,
}

impl fmt::Display for ProvenanceKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} / chain {} / {}",
            self.contract_id,
            self.chain_id.as_u64(),
            self.env,
        )
    }
}
