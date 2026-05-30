use std::collections::BTreeSet;
use std::path::Path;

use cow_sdk_orderbook::{
    AuctionOrder, OnchainOrderData, Order, OrderQuoteResponse, QuoteData, SolverExecution,
    StoredOrderQuote, TotalSurplus, Trade,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Deserialize)]
struct Coverage {
    version: u32,
    dtos: Vec<DtoCoverage>,
}

#[derive(Debug, Deserialize)]
struct DtoCoverage {
    schema: String,
    rust_type: String,
    inventory: String,
    required_fields: Vec<String>,
    fixtures: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Inventory {
    expanded_required: Vec<String>,
}

fn workspace_file(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn roundtrip_fixture<T>(path: &str)
where
    T: DeserializeOwned + Serialize,
{
    let raw = std::fs::read_to_string(workspace_file(path))
        .unwrap_or_else(|error| panic!("{path} must be readable: {error}"));
    let value: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|error| panic!("{path} must be JSON: {error}"));
    let typed: T = serde_json::from_value(value.clone()).unwrap_or_else(|error| {
        panic!("{path} must deserialize into the mapped Rust DTO: {error}")
    });
    let rendered = serde_json::to_value(&typed)
        .unwrap_or_else(|error| panic!("{path} must serialize from the mapped Rust DTO: {error}"));
    let expected = value
        .as_object()
        .unwrap_or_else(|| panic!("{path} fixture root must be a JSON object"));
    let actual = rendered
        .as_object()
        .unwrap_or_else(|| panic!("{path} serialized DTO root must be a JSON object"));
    for (field, expected_value) in expected {
        assert_eq!(
            actual.get(field),
            Some(expected_value),
            "{path}: serialized DTO must preserve fixture field {field}",
        );
    }
}

fn assert_fixture_roundtrips(entry: &DtoCoverage) {
    for fixture in &entry.fixtures {
        match entry.rust_type.as_str() {
            "cow_sdk_orderbook::Order" => roundtrip_fixture::<Order>(fixture),
            "cow_sdk_orderbook::AuctionOrder" => roundtrip_fixture::<AuctionOrder>(fixture),
            "cow_sdk_orderbook::OrderQuoteResponse" => {
                roundtrip_fixture::<OrderQuoteResponse>(fixture);
            }
            "cow_sdk_orderbook::QuoteData" => roundtrip_fixture::<QuoteData>(fixture),
            "cow_sdk_orderbook::Trade" => roundtrip_fixture::<Trade>(fixture),
            "cow_sdk_orderbook::StoredOrderQuote" => roundtrip_fixture::<StoredOrderQuote>(fixture),
            "cow_sdk_orderbook::OnchainOrderData" => {
                roundtrip_fixture::<OnchainOrderData>(fixture);
            }
            "cow_sdk_orderbook::TotalSurplus" => roundtrip_fixture::<TotalSurplus>(fixture),
            "cow_sdk_orderbook::SolverExecution" => {
                roundtrip_fixture::<SolverExecution>(fixture);
            }
            other => panic!("coverage.yaml maps unsupported Rust DTO type {other}"),
        }
    }
}

#[test]
fn openapi_coverage_manifest_roundtrips_required_orderbook_dtos() {
    let raw = std::fs::read_to_string(workspace_file("parity/openapi/coverage.yaml"))
        .expect("coverage manifest must be readable");
    let coverage: Coverage =
        serde_yaml::from_str(&raw).expect("coverage manifest must parse as YAML");

    assert_eq!(coverage.version, 1);

    let required = BTreeSet::from([
        "cow_sdk_orderbook::Order",
        "cow_sdk_orderbook::AuctionOrder",
        "cow_sdk_orderbook::OrderQuoteResponse",
        "cow_sdk_orderbook::QuoteData",
        "cow_sdk_orderbook::Trade",
        "cow_sdk_orderbook::StoredOrderQuote",
        "cow_sdk_orderbook::OnchainOrderData",
        "cow_sdk_orderbook::TotalSurplus",
        "cow_sdk_orderbook::SolverExecution",
    ]);
    let covered = coverage
        .dtos
        .iter()
        .map(|entry| entry.rust_type.as_str())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        covered, required,
        "coverage.yaml must enumerate exactly the reviewed orderbook DTO surface",
    );

    for entry in &coverage.dtos {
        assert!(
            workspace_file(&entry.inventory).is_file(),
            "{} inventory file must exist",
            entry.inventory,
        );
        let inventory_raw = std::fs::read_to_string(workspace_file(&entry.inventory))
            .unwrap_or_else(|error| {
                panic!("{} inventory must be readable: {error}", entry.inventory)
            });
        let inventory: Inventory = serde_yaml::from_str(&inventory_raw)
            .unwrap_or_else(|error| panic!("{} inventory must parse: {error}", entry.inventory));
        assert_eq!(
            entry.required_fields.iter().collect::<BTreeSet<_>>(),
            inventory.expanded_required.iter().collect::<BTreeSet<_>>(),
            "{} required_fields must match inventory expanded_required",
            entry.rust_type,
        );
        assert!(
            !entry.schema.trim().is_empty(),
            "{} must identify its OpenAPI schema path",
            entry.rust_type,
        );
        assert_fixture_roundtrips(entry);
    }
}
