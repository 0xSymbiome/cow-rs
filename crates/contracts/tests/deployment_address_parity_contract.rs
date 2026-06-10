#![cfg(feature = "cow-shed")]

//! Contract: the chain-keyed `cow_shed_factory` / `cow_shed_implementation`
//! lookups and `proxy_for` agree with the deployed-reality probe
//! (`version-call-results.json`) and the CREATE2 reference vectors
//! (`proxy_addresses.json`). This locks the per-chain address table against
//! drift, including the Gnosis Chain factory/implementation divergence.

use cow_sdk_contracts::DeploymentChainId;
use cow_sdk_contracts::cow_shed::{
    CowShedVersion, cow_shed_factory, cow_shed_implementation, proxy_for,
};
use serde::Deserialize;

mod cow_shed_common;
use cow_shed_common::{address, parse_version};

const VERSION_CALLS: &str = include_str!("fixtures/version-call-results.json");
const PROXY_ADDRESSES: &str =
    include_str!("../../../parity/fixtures/cow_shed/proxy_addresses.json");

#[derive(Deserialize)]
struct VersionCalls {
    version_calls: Vec<VersionCallRow>,
}

#[derive(Deserialize)]
struct VersionCallRow {
    chain_id: u64,
    factory: String,
    implementation: String,
    decoded_version: String,
}

#[derive(Deserialize)]
struct ProxyAddresses {
    rows: Vec<ProxyRow>,
}

#[derive(Deserialize)]
struct ProxyRow {
    version: String,
    chain_id: u64,
    factory: String,
    user: String,
    proxy: String,
}

#[test]
fn factory_and_implementation_match_the_deployed_probe() {
    let probe: VersionCalls =
        serde_json::from_str(VERSION_CALLS).expect("version-call-results.json parses");
    assert!(
        !probe.version_calls.is_empty(),
        "version-call-results must carry per-chain rows"
    );

    for row in &probe.version_calls {
        let Ok(chain) = DeploymentChainId::try_from(row.chain_id) else {
            continue;
        };
        let version = parse_version(&row.decoded_version);
        assert_eq!(
            cow_shed_factory(chain, version),
            address(&row.factory),
            "factory lookup diverges from probe for chain {}",
            row.chain_id
        );
        assert_eq!(
            cow_shed_implementation(chain, version),
            address(&row.implementation),
            "implementation lookup diverges from probe for chain {}",
            row.chain_id
        );
    }
}

#[test]
fn proxy_for_matches_reference_vectors() {
    let fixture: ProxyAddresses =
        serde_json::from_str(PROXY_ADDRESSES).expect("proxy_addresses.json parses");
    assert!(!fixture.rows.is_empty(), "proxy address vectors must exist");

    for row in &fixture.rows {
        let Ok(chain) = DeploymentChainId::try_from(row.chain_id) else {
            continue;
        };
        let version = parse_version(&row.version);
        assert_eq!(
            cow_shed_factory(chain, version),
            address(&row.factory),
            "factory mismatch for chain {} version {}",
            row.chain_id,
            row.version
        );
        assert_eq!(
            proxy_for(chain, version, address(&row.user)),
            address(&row.proxy),
            "proxy_for mismatch for chain {} version {} user {}",
            row.chain_id,
            row.version,
            row.user
        );
    }
}

/// The chain-keyed lookups accept a [`cow_sdk_core::SupportedChainId`] (what a
/// trading flow already holds) and resolve to the same addresses as the
/// [`DeploymentChainId`] it converts into — including the Gnosis divergence.
/// This locks the `impl Into<DeploymentChainId>` ergonomic contract.
#[test]
fn supported_chain_id_bridges_to_the_same_addresses() {
    use cow_sdk_core::SupportedChainId;

    let user = address("0x76b0340e50BD9883D8B2CA5fd9f52439a9e7Cf58");
    for version in [CowShedVersion::V1_0_0, CowShedVersion::V1_0_1] {
        for (supported, deployment) in [
            (SupportedChainId::Mainnet, DeploymentChainId::Mainnet),
            (
                SupportedChainId::GnosisChain,
                DeploymentChainId::GnosisChain,
            ),
            (
                SupportedChainId::ArbitrumOne,
                DeploymentChainId::ArbitrumOne,
            ),
        ] {
            assert_eq!(
                cow_shed_factory(supported, version),
                cow_shed_factory(deployment, version),
                "factory diverges between chain-id types ({supported:?}, {version})"
            );
            assert_eq!(
                cow_shed_implementation(supported, version),
                cow_shed_implementation(deployment, version),
                "implementation diverges between chain-id types ({supported:?}, {version})"
            );
            assert_eq!(
                proxy_for(supported, version, user),
                proxy_for(deployment, version, user),
                "proxy_for diverges between chain-id types ({supported:?}, {version})"
            );
        }
    }

    // Gnosis is the special case: its v1.0.1 factory differs from the canonical
    // one, so the chain-id bridge must keep the two distinct.
    assert_ne!(
        cow_shed_factory(SupportedChainId::GnosisChain, CowShedVersion::V1_0_1),
        cow_shed_factory(SupportedChainId::Mainnet, CowShedVersion::V1_0_1),
        "Gnosis v1.0.1 factory must differ from the canonical factory"
    );
}
