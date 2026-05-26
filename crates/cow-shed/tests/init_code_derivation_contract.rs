use alloy_primitives::{Address, B256};
use cow_sdk_cow_shed::CowShedVersion;
use cow_sdk_cow_shed::address::{implementation_for, init_code_hash, proxy_of};
use serde::Deserialize;

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

fn parse_version(value: &str) -> CowShedVersion {
    match value {
        "1.0.0" => CowShedVersion::V1_0_0,
        "1.0.1" => CowShedVersion::V1_0_1,
        other => panic!("unsupported fixture version {other}"),
    }
}

fn b256(value: &str) -> B256 {
    let bytes =
        alloy_primitives::hex::decode(value.trim_start_matches("0x")).expect("fixture hash parses");
    let mut out = [0_u8; 32];
    out.copy_from_slice(&bytes);
    B256::from(out)
}

fn address(value: &str) -> Address {
    value.parse().expect("fixture address parses")
}
