//! Contract suite for the DTO surface that genuinely crosses a boundary.
//!
//! Most contract DTOs reach the chain through ABI encoders or are serialized as
//! tuples, so their derived `serde` JSON is incidental and is not pinned here.
//! This suite pins the cases that are real contracts:
//!
//! - [`OrderCancellations`] — the live `DELETE /api/v1/orders` request body.
//! - eth-flow `createOrder` — the on-chain calldata byte layout, cross-checked
//!   against an independent local `sol!` re-encoding (a differential oracle).
//! - the `#[non_exhaustive]` marker policy (ADR 0027) across the signing enums,
//!   driven by `.github/config/enum-policy.yaml`.

use alloy_sol_types::{
    SolCall,
    private::{Address as SolAddress, FixedBytes, U256},
    sol,
};
use cow_sdk_contracts::{EthFlowOrderData, OrderCancellations, encode_create_order_calldata};
use cow_sdk_core::{Amount, AppDataHash, OrderUid};
use cow_sdk_test_utils::builders::address;
use serde::Deserialize;
use serde::Serialize;

const CONTRACT_SIGNATURE_SOURCE: &str = include_str!("../src/signature.rs");
const ORDERBOOK_TYPES_SOURCE: &str = include_str!("../../orderbook/src/types/enums.rs");

const ADDR1: &str = "0x1111111111111111111111111111111111111111";
const ADDR2: &str = "0x2222222222222222222222222222222222222222";
const APP_DATA: &str = concat!(
    "0x", "aaaaaaaa", "aaaaaaaa", "aaaaaaaa", "aaaaaaaa", "aaaaaaaa", "aaaaaaaa", "aaaaaaaa",
    "aaaaaaaa",
);
const UID1: &str = concat!(
    "0x", "dddddddd", "dddddddd", "dddddddd", "dddddddd", "dddddddd", "dddddddd", "dddddddd",
    "dddddddd", "eeeeeeee", "eeeeeeee", "eeeeeeee", "eeeeeeee", "eeeeeeee", "0000002a",
);
const UID2: &str = concat!(
    "0x", "ffffffff", "ffffffff", "ffffffff", "ffffffff", "ffffffff", "ffffffff", "ffffffff",
    "ffffffff", "11111111", "11111111", "11111111", "11111111", "11111111", "0000002b",
);

sol! {
    #[sol(rename_all = "camelcase")]
    interface LocalEthFlow {
        struct EthFlowOrderData {
            address buyToken;
            address receiver;
            uint256 sellAmount;
            uint256 buyAmount;
            bytes32 appData;
            uint256 feeAmount;
            uint32 validTo;
            bool partiallyFillable;
            int64 quoteId;
        }

        function createOrder(EthFlowOrderData calldata order) external payable;
    }
}

fn assert_json_bytes<T>(value: &T, expected: &str)
where
    T: Serialize,
{
    let actual = serde_json::to_string(value).expect("DTO serialization must succeed");
    assert_eq!(actual, expected);
}

fn amount(value: &str) -> Amount {
    Amount::new(value).expect("amount literal must stay valid")
}

fn app_data() -> AppDataHash {
    AppDataHash::new(APP_DATA).expect("app-data literal must stay valid")
}

fn order_uid(value: &str) -> OrderUid {
    OrderUid::new(value).expect("order UID literal must stay valid")
}

#[test]
fn order_cancellations_new_preserves_wire_shape() {
    let cancellations = OrderCancellations::new(vec![order_uid(UID1), order_uid(UID2)]);
    let expected = format!("{{\"orderUids\":[\"{UID1}\",\"{UID2}\"]}}");

    assert_json_bytes(&cancellations, &expected);
}

#[test]
fn eth_flow_order_data_new_preserves_abi_shape() {
    let order = EthFlowOrderData::new(
        address(ADDR1),
        address(ADDR2),
        amount("15"),
        amount("16"),
        app_data(),
        Amount::ZERO,
        77,
        false,
        7,
    )
    .expect("non-zero receiver fixture must construct successfully");

    let actual = encode_create_order_calldata(&order);
    let expected = LocalEthFlow::createOrderCall {
        order: LocalEthFlow::EthFlowOrderData {
            buyToken: SolAddress::from([0x11; 20]),
            receiver: SolAddress::from([0x22; 20]),
            sellAmount: U256::from(15_u64),
            buyAmount: U256::from(16_u64),
            appData: FixedBytes::from([0xaa; 32]),
            feeAmount: U256::ZERO,
            validTo: 77,
            partiallyFillable: false,
            quoteId: 7,
        },
    }
    .abi_encode();

    assert_eq!(actual, expected);
}

#[test]
fn adr_0027_signature_family_non_exhaustive() {
    assert_enum_has_non_exhaustive(CONTRACT_SIGNATURE_SOURCE, "SigningScheme");
    assert_enum_has_non_exhaustive(CONTRACT_SIGNATURE_SOURCE, "Signature");
    assert_enum_has_non_exhaustive(ORDERBOOK_TYPES_SOURCE, "SigningScheme");
}

#[test]
fn enum_policy_manifest_entries_match_expected_markers() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("contracts crate must live under crates/contracts");
    let manifest_path = repo_root.join(".github/config/enum-policy.yaml");
    let manifest = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", manifest_path.display()));
    let policy: EnumPolicyManifest =
        serde_yaml::from_str(&manifest).expect("enum-policy.yaml must parse");

    assert_eq!(
        policy.version, 1,
        "enum-policy schema version must stay at 1"
    );
    assert!(
        !policy.enums.is_empty(),
        "enum-policy.yaml must classify at least one enum",
    );

    for entry in policy.enums {
        if entry.planned {
            // Reserved entry: the source-of-truth Rust definition lands in a
            // later capability landing, so skip the file/line marker check
            // until the enum exists.
            continue;
        }
        let source_path = repo_root.join(&entry.file);
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));
        let has_marker = enum_has_non_exhaustive(&source, &entry.name);
        match entry.expected_marker.as_str() {
            "non_exhaustive" => assert!(
                has_marker,
                "{} in {} must carry #[non_exhaustive]",
                entry.name, entry.file,
            ),
            "exhaustive" => assert!(
                !has_marker,
                "{} in {} must not carry #[non_exhaustive]",
                entry.name, entry.file,
            ),
            marker => panic!(
                "{} in enum-policy.yaml has unsupported expected_marker `{marker}`",
                entry.name,
            ),
        }
    }
}

#[derive(Deserialize)]
struct EnumPolicyManifest {
    version: u32,
    enums: Vec<EnumPolicyEntry>,
}

#[derive(Deserialize)]
struct EnumPolicyEntry {
    name: String,
    file: String,
    expected_marker: String,
    #[serde(default)]
    planned: bool,
}

fn assert_enum_has_non_exhaustive(source: &str, enum_name: &str) {
    assert!(
        enum_has_non_exhaustive(source, enum_name),
        "public enum `{enum_name}` must carry #[non_exhaustive]",
    );
}

fn enum_has_non_exhaustive(source: &str, enum_name: &str) -> bool {
    let enum_start = source
        .find(&format!("pub enum {enum_name}"))
        .unwrap_or_else(|| panic!("public enum `{enum_name}` must exist"));
    let preceding = &source[..enum_start];
    let item_header_start = preceding
        .rfind("\n///")
        .or_else(|| preceding.rfind("\n#["))
        .map_or(0, |position| position + 1);
    let item_header = &preceding[item_header_start..];

    item_header.contains("#[non_exhaustive]")
}
