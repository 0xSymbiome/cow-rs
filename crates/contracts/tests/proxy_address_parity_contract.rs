#![cfg(feature = "cow-shed")]

//! Contract: CREATE2 proxy derivation reproduces the reference vectors in
//! `cow_shed/proxy_addresses.json`. The anchor rows are the TS arbiter's own
//! golden vectors (external anchors that transitively prove the embedded
//! creation-code bytes and the formula); the embedded blobs themselves are
//! pinned by byte length + keccak256, byte-identical to the arbiter's
//! `COW_SHED_PROXY_INIT_CODE` constants at the pinned commit.

use alloy_primitives::{B256, keccak256};
use cow_sdk_contracts::cow_shed::{
    cow_shed_factory, cow_shed_implementation, init_code_hash, proxy_for, proxy_of,
};
use serde::Deserialize;

mod cow_shed_common;
use cow_shed_common::{address, b256, parse_version};

const FIXTURE: &str = include_str!("../../../parity/fixtures/cow_shed/proxy_addresses.json");
const V1_0_0_CODE: &[u8] = include_bytes!("../src/cow_shed/address/proxy-creation-code/v1.0.0.bin");
const V1_0_1_CODE: &[u8] = include_bytes!("../src/cow_shed/address/proxy-creation-code/v1.0.1.bin");

#[derive(Debug, Deserialize)]
struct Fixture {
    proxy_creation_code: CreationCode,
    rows: Vec<Row>,
}

#[derive(Debug, Deserialize)]
struct CreationCode {
    #[serde(rename = "1.0.0")]
    v1_0_0: Digests,
    #[serde(rename = "1.0.1")]
    v1_0_1: Digests,
}

#[derive(Debug, Deserialize)]
struct Digests {
    byte_len: usize,
    keccak256: String,
}

#[derive(Debug, Deserialize)]
struct Row {
    version: String,
    canonical: bool,
    anchor: Option<String>,
    factory: String,
    implementation: String,
    user: String,
    salt: String,
    init_code_hash: String,
    proxy: String,
}

#[test]
fn creation_code_blobs_are_digest_pinned() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("proxy fixture parses");
    for (blob, digests) in [
        (V1_0_0_CODE, &fixture.proxy_creation_code.v1_0_0),
        (V1_0_1_CODE, &fixture.proxy_creation_code.v1_0_1),
    ] {
        assert_eq!(blob.len(), digests.byte_len, "creation code byte length");
        assert_eq!(
            keccak256(blob),
            b256(&digests.keccak256),
            "creation code keccak256"
        );
    }
}

#[test]
fn rows_reproduce_via_init_code_hash_and_create2() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("proxy fixture parses");
    assert!(!fixture.rows.is_empty(), "proxy address vectors must exist");

    let mut anchors = 0_usize;
    for row in &fixture.rows {
        let version = parse_version(&row.version);
        let factory = address(&row.factory);
        let implementation = address(&row.implementation);
        let user = address(&row.user);
        let proxy = address(&row.proxy);

        assert_eq!(
            user.into_word(),
            b256(&row.salt),
            "row {}: the CREATE2 salt is the user address as a 32-byte word",
            row.user
        );
        let hash = init_code_hash(version, implementation, user);
        assert_eq!(
            B256::from(hash),
            b256(&row.init_code_hash),
            "row {}: init-code hash",
            row.user
        );
        assert_eq!(
            factory.create2(user.into_word(), B256::from(hash)),
            proxy,
            "row {}: CREATE2 proxy",
            row.user
        );

        if row.canonical {
            assert_eq!(
                factory,
                cow_shed_factory(version),
                "canonical rows use the version's canonical factory"
            );
            assert_eq!(
                implementation,
                cow_shed_implementation(version),
                "canonical rows use the version's canonical implementation"
            );
            assert_eq!(proxy_of(version, factory, user), proxy);
            assert_eq!(
                proxy_for(version, user),
                proxy,
                "row {}: proxy_for is chain-independent and canonical",
                row.user
            );
        }

        if row.anchor.is_some() {
            anchors += 1;
        }
    }

    assert!(
        anchors >= 2,
        "the TS arbiter anchor vectors (canonical golden + custom-options mock) must stay present"
    );
}
