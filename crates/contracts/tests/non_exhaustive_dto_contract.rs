//! Contract suite for the DTO surface that genuinely crosses a boundary.
//!
//! Most contract DTOs reach the chain through ABI encoders or are serialized as
//! tuples, so their derived `serde` JSON is incidental and is not pinned here.
//! This suite pins the cases that are real contracts:
//!
//! - [`OrderCancellations`] — the live `DELETE /api/v1/orders` request body.
//! - eth-flow `createOrder` — the on-chain calldata byte layout, cross-checked
//!   against an independent local `sol!` re-encoding (a differential oracle).
//!
//! The `#[non_exhaustive]` marker policy (ADR 0027) is enforced workspace-wide by
//! the syn-based `xtask check-enum-policy` gate over `.github/config/enum-policy.yaml`,
//! so it is not re-checked here.

use alloy_sol_types::{
    SolCall,
    private::{Address as SolAddress, FixedBytes, U256},
    sol,
};
use cow_sdk_contracts::{EthFlowOrderData, OrderCancellations, encode_create_order_calldata};
use cow_sdk_core::{Amount, AppDataHash, OrderUid};
use cow_sdk_test_utils::builders::address;
use serde::Serialize;

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
