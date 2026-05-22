//! `GPv2` `Order` EIP-712 hashing parity contract.
//!
//! Drives the eight representative rows in
//! `parity/fixtures/eip712/order_digests.json` against the canonical
//! `alloy_sol_types::SolStruct::eip712_signing_hash` path exposed by
//! [`cow_sdk_contracts::GPv2Order`]. Each row pins the per-domain
//! separator, the per-order struct hash, and the final signing hash so a
//! future change to the EIP-712 typed-data encoding cannot silently
//! move the wire bytes.

use std::str::FromStr;

use alloy_primitives::{Address as AlloyAddress, B256, U256};
use alloy_sol_types::{Eip712Domain, SolStruct};
use cow_sdk_contracts::GPv2Order;
use serde::Deserialize;

const FIXTURE: &str = include_str!("../../../parity/fixtures/eip712/order_digests.json");

#[derive(Debug, Deserialize)]
struct Fixture {
    schema_version: u32,
    order_type_hash: String,
    rows: Vec<Row>,
}

#[derive(Debug, Deserialize)]
struct Row {
    name: String,
    domain: DomainCase,
    order: OrderCase,
    expected: Expected,
}

#[derive(Debug, Deserialize)]
struct DomainCase {
    name: String,
    version: String,
    chain_id: u64,
    verifying_contract: String,
}

#[derive(Debug, Deserialize)]
#[expect(
    non_snake_case,
    reason = "fields mirror the upstream services JSON keys in camelCase form so the serde deserialization matches the on-disk parity fixture row layout"
)]
struct OrderCase {
    sellToken: String,
    buyToken: String,
    receiver: String,
    sellAmount: String,
    buyAmount: String,
    validTo: u32,
    appData: String,
    feeAmount: String,
    kind: String,
    partiallyFillable: bool,
    sellTokenBalance: String,
    buyTokenBalance: String,
}

#[derive(Debug, Deserialize)]
struct Expected {
    domain_separator: String,
    order_struct_hash: String,
    signing_hash: String,
}

#[test]
fn order_digest_fixture_rows_hold() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("order digest fixture parses");
    assert_eq!(fixture.schema_version, 1, "fixture schema version pinned");
    assert_eq!(
        fixture.order_type_hash,
        format!(
            "0x{}",
            hex::encode(GPv2Order::default().eip712_type_hash().as_slice())
        ),
        "fixture order_type_hash matches GPv2Order::eip712_type_hash",
    );
    assert_eq!(
        fixture.rows.len(),
        8,
        "fixture carries 8 representative rows"
    );

    for row in &fixture.rows {
        let alloy_domain = Eip712Domain {
            name: Some(row.domain.name.clone().into()),
            version: Some(row.domain.version.clone().into()),
            chain_id: Some(U256::from(row.domain.chain_id)),
            verifying_contract: Some(parse_address(&row.domain.verifying_contract)),
            salt: None,
        };
        let order = build_order(&row.order);

        let actual_domain_separator = format!("{}", alloy_domain.separator());
        assert_eq!(
            actual_domain_separator, row.expected.domain_separator,
            "row {}: domain separator must match the fixture",
            row.name
        );

        let actual_struct_hash = format!("{}", order.eip712_hash_struct());
        assert_eq!(
            actual_struct_hash, row.expected.order_struct_hash,
            "row {}: order struct hash must match the fixture",
            row.name
        );

        let actual_signing_hash = format!("{}", order.eip712_signing_hash(&alloy_domain));
        assert_eq!(
            actual_signing_hash, row.expected.signing_hash,
            "row {}: signing hash must match the fixture",
            row.name
        );
    }
}

fn build_order(case: &OrderCase) -> GPv2Order {
    GPv2Order {
        sellToken: parse_address(&case.sellToken),
        buyToken: parse_address(&case.buyToken),
        receiver: parse_address(&case.receiver),
        sellAmount: parse_u256(&case.sellAmount),
        buyAmount: parse_u256(&case.buyAmount),
        validTo: case.validTo,
        appData: parse_b256(&case.appData),
        feeAmount: parse_u256(&case.feeAmount),
        kind: case.kind.clone(),
        partiallyFillable: case.partiallyFillable,
        sellTokenBalance: case.sellTokenBalance.clone(),
        buyTokenBalance: case.buyTokenBalance.clone(),
    }
}

fn parse_address(value: &str) -> AlloyAddress {
    AlloyAddress::from_str(value).expect("fixture address parses")
}

fn parse_b256(value: &str) -> B256 {
    B256::from_str(value).expect("fixture b256 parses")
}

fn parse_u256(value: &str) -> U256 {
    U256::from_str_radix(value, 10).expect("fixture u256 parses")
}
