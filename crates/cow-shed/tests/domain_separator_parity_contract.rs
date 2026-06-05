use cow_sdk_cow_shed::cow_shed_domain_separator;
use serde::Deserialize;

mod common;
use common::{address, b256, parse_version};

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
