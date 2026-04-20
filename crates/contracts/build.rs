//! Compile-time validator for the chain-keyed deployment registry manifest.
//!
//! Parses `crates/contracts/registry.toml` through the same TOML dialect the
//! runtime loader uses and rejects any row that violates the reviewed
//! invariants with a precise diagnostic that names the offending manifest
//! line so operators see an actionable fix target rather than a generic
//! parse error.

use std::collections::BTreeSet;
use std::path::Path;

use serde::Deserialize;

// The authoritative chain-id set is declared in a single shared include
// that the `src/` tree consumes as a Rust module. The build script cannot
// depend on the same crate's compiled output without inviting a circular
// build, so the include! brings the same literal into the build context.
include!("src/chain_ids.rs");

const SCHEMA_VERSION: u32 = 1;
const MANIFEST_PATH: &str = "registry.toml";

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
    println!("cargo:rerun-if-changed=src/chain_ids.rs");

    let path = Path::new(MANIFEST_PATH);
    let raw = std::fs::read_to_string(path).unwrap_or_else(|source| {
        let display = path.display();
        fail(&format!("failed to read `{display}`: {source}"));
    });

    let manifest: ManifestSchema = match toml::from_str(&raw) {
        Ok(parsed) => parsed,
        Err(error) => {
            fail(&format!("{MANIFEST_PATH}: malformed TOML — {error}"));
        }
    };

    if manifest.schema_version != SCHEMA_VERSION {
        let actual = manifest.schema_version;
        fail(&format!(
            "{MANIFEST_PATH}: unsupported schema_version {actual}; expected {SCHEMA_VERSION}",
        ));
    }

    let supported: BTreeSet<u64> = SUPPORTED_CHAIN_IDS.iter().copied().collect();
    let mut seen: BTreeSet<(String, u64, String)> = BTreeSet::new();

    for (index, entry) in manifest.entries.iter().enumerate() {
        let row = index + 1;

        match entry.contract_id.as_str() {
            "Settlement" | "VaultRelayer" | "EthFlow" => {}
            other => fail(&format!(
                "{MANIFEST_PATH}: entry #{row} declares unknown contract_id `{other}`; expected one of Settlement, VaultRelayer, EthFlow",
            )),
        }

        match entry.env.as_str() {
            "prod" | "staging" => {}
            other => fail(&format!(
                "{MANIFEST_PATH}: entry #{row} declares unknown env `{other}`; expected prod or staging",
            )),
        }

        if !supported.contains(&entry.chain_id) {
            let chain_id = entry.chain_id;
            fail(&format!(
                "{MANIFEST_PATH}: entry #{row} declares unsupported chain_id {chain_id}; expected one of the eleven supported chain ids",
            ));
        }

        if !is_valid_ethereum_address(&entry.address) {
            let address = &entry.address;
            fail(&format!(
                "{MANIFEST_PATH}: entry #{row} declares malformed address `{address}` — expected a 0x-prefixed 40-character hex literal",
            ));
        }

        let key = (entry.contract_id.clone(), entry.chain_id, entry.env.clone());
        if !seen.insert(key) {
            let contract_id = &entry.contract_id;
            let chain_id = entry.chain_id;
            let env = &entry.env;
            fail(&format!(
                "{MANIFEST_PATH}: entry #{row} duplicates (contract_id=`{contract_id}`, chain_id={chain_id}, env=`{env}`)",
            ));
        }
    }
}

fn is_valid_ethereum_address(candidate: &str) -> bool {
    let Some(body) = candidate.strip_prefix("0x") else {
        return false;
    };
    body.len() == 40 && body.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn fail(message: &str) -> ! {
    // Emit a cargo-visible error line so the offending manifest detail is
    // surfaced through every cargo invocation, then halt the build so a
    // malformed registry cannot land silently.
    eprintln!("error: {message}");
    std::process::exit(1);
}
