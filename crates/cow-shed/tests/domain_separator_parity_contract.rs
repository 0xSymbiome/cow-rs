use alloy_primitives::{Address, B256};
use cow_sdk_cow_shed::{CowShedVersion, cow_shed_domain_separator};
use serde::Deserialize;

const FIXTURE: &str = include_str!("../../../parity/fixtures/cow_shed/domain_separator.json");

#[derive(Debug, Deserialize)]
struct Fixture {
    rows: Vec<Row>,
}

#[derive(Debug, Deserialize)]
struct Row {
    chain_id: u64,
    version: String,
    proxy: String,
    domain_separator: String,
}

#[test]
fn domain_separators_match_reference_vectors() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("domain fixture parses");
    for row in fixture.rows {
        let actual = cow_shed_domain_separator(
            row.chain_id,
            parse_version(&row.version),
            address(&row.proxy),
        );
        assert_eq!(actual, b256(&row.domain_separator));
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
    let bytes = hex::decode(value.trim_start_matches("0x")).expect("fixture hash parses");
    let mut out = [0_u8; 32];
    out.copy_from_slice(&bytes);
    B256::from(out)
}

fn address(value: &str) -> Address {
    value.parse().expect("fixture address parses")
}
