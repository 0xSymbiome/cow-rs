#![cfg(feature = "cow-shed")]

use alloy_primitives::B256;
use cow_sdk_contracts::cow_shed::address::{implementation_for, init_code_hash, proxy_of};
use serde::Deserialize;

mod cow_shed_common;
use cow_shed_common::{address, b256, parse_version};

const FIXTURE: &str = include_str!("../../../parity/fixtures/cow_shed/proxy_addresses.json");

#[derive(Debug, Deserialize)]
struct Fixture {
    rows: Vec<Row>,
}

#[derive(Debug, Deserialize)]
struct Row {
    version: String,
    factory: String,
    implementation: String,
    user: String,
    salt: String,
    init_code_hash: String,
    proxy: String,
}

#[test]
fn init_code_is_derived_per_implementation_and_user() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("proxy fixture parses");
    for row in fixture.rows {
        let version = parse_version(&row.version);
        let factory = address(&row.factory);
        let implementation = address(&row.implementation);
        let user = address(&row.user);
        assert_eq!(implementation_for(version, factory), implementation);
        assert_eq!(user.into_word(), b256(&row.salt));
        assert_eq!(
            B256::from(init_code_hash(version, implementation, user)),
            b256(&row.init_code_hash)
        );
        assert_eq!(proxy_of(version, factory, user), address(&row.proxy));
    }
}
