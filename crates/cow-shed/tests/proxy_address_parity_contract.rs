use cow_sdk_cow_shed::proxy_of;
use serde::Deserialize;

mod common;
use common::{address, parse_version};

const FIXTURE: &str = include_str!("../../../parity/fixtures/cow_shed/proxy_addresses.json");
const ANCHOR_USER: &str = "0x76b0340e50BD9883D8B2CA5fd9f52439a9e7Cf58";
const ANCHOR_PROXY: &str = "0x66545B93A314e5BdEC9E5Ff9c4D2C7054e6afb04";

#[derive(Debug, Deserialize)]
struct Fixture {
    rows: Vec<Row>,
}

#[derive(Debug, Deserialize)]
struct Row {
    version: String,
    factory: String,
    user: String,
    proxy: String,
}

#[test]
fn proxy_addresses_match_reference_vectors() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("proxy fixture parses");
    assert!(
        fixture.rows.len() >= 10,
        "proxy fixture must carry enough parity rows"
    );

    let mut anchor_seen = false;
    for row in fixture.rows {
        let version = parse_version(&row.version);
        let factory = address(&row.factory);
        let user = address(&row.user);
        let proxy = address(&row.proxy);
        let actual = proxy_of(version, factory, user);
        assert_eq!(
            actual, proxy,
            "proxy mismatch for version {} factory {:#x} user {:#x}",
            row.version, factory, user
        );

        if row.version == "1.0.1" && user == address(ANCHOR_USER) && proxy == address(ANCHOR_PROXY)
        {
            anchor_seen = true;
        }
    }

    assert!(anchor_seen, "canonical mainnet v1.0.1 proxy anchor missing");
}
