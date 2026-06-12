#![cfg(feature = "cow-shed")]

//! Contract: the version-keyed `cow_shed_factory` / `cow_shed_implementation`
//! lookups and the EIP-712 version strings agree with the deployed-generation
//! record (`cow_shed/deployments.json`). Each pair is a deterministic CREATE2
//! deployment, identical on every supported chain, so the record carries no
//! chain axis; the CREATE2 reference vectors live in
//! `cow_shed/proxy_addresses.json`.

use cow_sdk_contracts::cow_shed::{CowShedVersion, cow_shed_factory, cow_shed_implementation};
use serde::Deserialize;

mod cow_shed_common;
use cow_shed_common::{address, parse_version};

const DEPLOYMENTS: &str = include_str!("../../../parity/fixtures/cow_shed/deployments.json");

#[derive(Deserialize)]
struct Deployments {
    deployments: Vec<Row>,
}

#[derive(Deserialize)]
struct Row {
    version: String,
    factory: String,
    implementation: String,
    eip712_version: String,
}

#[test]
fn version_keyed_lookups_match_the_deployment_record() {
    let fixture: Deployments = serde_json::from_str(DEPLOYMENTS).expect("deployments.json parses");
    assert_eq!(
        fixture.deployments.len(),
        CowShedVersion::ALL.len(),
        "one deployment row per supported version"
    );

    for row in &fixture.deployments {
        let version = parse_version(&row.version);
        assert_eq!(
            cow_shed_factory(version),
            address(&row.factory),
            "factory lookup diverges from the deployment record for {version}"
        );
        assert_eq!(
            cow_shed_implementation(version),
            address(&row.implementation),
            "implementation lookup diverges from the deployment record for {version}"
        );
        assert_eq!(
            version.version_str(),
            row.eip712_version,
            "EIP-712 domain version diverges from the deployed VERSION constant for {version}"
        );
    }
}
