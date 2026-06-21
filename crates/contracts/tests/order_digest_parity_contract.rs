//! `GPv2` order EIP-712 signing-hash parity contract.
//!
//! Drives the eight representative rows in
//! `parity/fixtures/eip712/order_digests.json` through the crate's public
//! [`cow_sdk_contracts::hash_order`] entrypoint so the canonical
//! `keccak256(0x19 || 0x01 || domain_separator || struct_hash)` order
//! signing hash stays byte-stable across mainnet, Gnosis Chain, Sepolia,
//! Arbitrum One, and Base. Driving `hash_order` (rather than the generated
//! codec directly) anchors the regression on the real enum-to-string and
//! EIP-712 composition path the SDK ships rather than on the underlying
//! `alloy` codec call. The fixture header type hash is checked against
//! [`cow_sdk_contracts::order_eip712_type_hash`].

use cow_sdk_contracts::{hash_order, order_eip712_type_hash};
use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderData, OrderKind, SellTokenSource,
    TypedDataDomain,
};
use serde::Deserialize;

const FIXTURE: &str = include_str!("../../../parity/fixtures/eip712/order_digests.json");

#[derive(Debug, Deserialize)]
struct Fixture {
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
    signing_hash: String,
}

#[test]
fn order_digest_fixture_rows_hold() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("order digest fixture parses");
    assert_eq!(
        fixture.order_type_hash,
        order_eip712_type_hash().to_hex_string(),
        "fixture order_type_hash matches the public order_eip712_type_hash()",
    );
    assert_eq!(
        fixture.rows.len(),
        8,
        "fixture carries 8 representative rows"
    );

    for row in &fixture.rows {
        let domain = TypedDataDomain::new(
            row.domain.name.clone(),
            row.domain.version.clone(),
            row.domain.chain_id,
            parse_address(&row.domain.verifying_contract),
        );
        let order = build_order(&row.order);

        let actual_signing_hash = hash_order(&domain, &order).to_hex_string();
        assert_eq!(
            actual_signing_hash, row.expected.signing_hash,
            "row {}: order signing hash must match the fixture",
            row.name
        );
    }
}

fn build_order(case: &OrderCase) -> OrderData {
    OrderData::new(
        parse_address(&case.sellToken),
        parse_address(&case.buyToken),
        parse_address(&case.receiver),
        parse_amount(&case.sellAmount),
        parse_amount(&case.buyAmount),
        case.validTo,
        parse_app_data(&case.appData),
        parse_amount(&case.feeAmount),
        parse_kind(&case.kind),
        case.partiallyFillable,
        parse_sell_balance(&case.sellTokenBalance),
        parse_buy_balance(&case.buyTokenBalance),
    )
}

fn parse_address(value: &str) -> Address {
    Address::new(value).expect("fixture address parses")
}

fn parse_amount(value: &str) -> Amount {
    Amount::new(value).expect("fixture amount parses")
}

fn parse_app_data(value: &str) -> AppDataHash {
    AppDataHash::new(value).expect("fixture app data hash parses")
}

fn parse_kind(value: &str) -> OrderKind {
    match value {
        "sell" => OrderKind::Sell,
        "buy" => OrderKind::Buy,
        other => panic!("unexpected order kind in fixture: {other}"),
    }
}

fn parse_sell_balance(value: &str) -> SellTokenSource {
    match value {
        "erc20" => SellTokenSource::Erc20,
        "external" => SellTokenSource::External,
        "internal" => SellTokenSource::Internal,
        other => panic!("unexpected sell token balance in fixture: {other}"),
    }
}

fn parse_buy_balance(value: &str) -> BuyTokenDestination {
    match value {
        "erc20" => BuyTokenDestination::Erc20,
        "internal" => BuyTokenDestination::Internal,
        other => panic!("unexpected buy token balance in fixture: {other}"),
    }
}
