#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::derive_partial_eq_without_eq,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, style, and perf lints acceptable in test helper code"
)]

mod common;

use sha3::{Digest, Keccak256};

use bytes::Bytes;
use cow_sdk_contracts::{
    AllowListReader, InteractionStage, SettlementReader, TradeSimulation,
    TradeSimulationBalanceDelta, TradeSimulationResult, TradeSimulator,
};
use cow_sdk_core::{Address, Amount, BuyTokenDestination, OrderUid, SignedAmount};

use common::{MockProvider, fixture_case};

fn balance_id(name: &str) -> String {
    let digest = Keccak256::digest(name.as_bytes());
    format!("0x{}", hex::encode(digest))
}

#[test]
fn reader_helpers_match_fixture_surface_and_encode_storage_requests() {
    let fixture = fixture_case("contracts-reader-helper-surface");
    let helpers = fixture["expected"]["helpers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        helpers,
        vec!["AllowListReader", "SettlementReader", "TradeSimulator"]
    );

    let provider = MockProvider::new();
    provider.set_response("true");

    let allow_list = AllowListReader {
        allow_list_address: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        allow_list_abi_json: serde_json::to_string(&["function areSolvers(address[])"]).unwrap(),
        reader_address: Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        reader_abi_json: serde_json::to_string(&["function areSolvers(address[])"]).unwrap(),
        provider: provider.clone(),
    };

    let solvers = vec![
        Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        Address::new("0x4444444444444444444444444444444444444444").unwrap(),
    ];
    assert!(allow_list.are_solvers(&solvers).unwrap());

    let call = provider.calls.borrow().last().cloned().unwrap();
    assert_eq!(call.method, "areSolvers");
    let payload: serde_json::Value = serde_json::from_str(&call.args_json).unwrap();
    assert_eq!(
        payload["baseAddress"],
        serde_json::json!(allow_list.allow_list_address)
    );
    assert_eq!(payload["method"], "areSolvers");
    assert_eq!(payload["parameters"], serde_json::json!(solvers));
}

#[test]
fn settlement_reader_and_trade_simulator_decode_typed_results() {
    let provider = MockProvider::new();
    provider.set_response("[\"1\",\"2\"]");

    let settlement_reader = SettlementReader {
        settlement_address: Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap(),
        settlement_abi_json: serde_json::to_string(&["function filledAmountsForOrders(bytes[])"])
            .unwrap(),
        reader_address: Address::new("0x5555555555555555555555555555555555555555").unwrap(),
        reader_abi_json: serde_json::to_string(&["function filledAmountsForOrders(bytes[])"])
            .unwrap(),
        provider: provider.clone(),
    };
    let order_uids = vec![
        OrderUid::new(format!(
            "0x{}{}{}",
            "01".repeat(32),
            "02".repeat(20),
            "00000000"
        ))
        .unwrap(),
        OrderUid::new(format!(
            "0x{}{}{}",
            "03".repeat(32),
            "04".repeat(20),
            "00000000"
        ))
        .unwrap(),
    ];
    assert_eq!(
        settlement_reader
            .filled_amounts_for_orders(&order_uids)
            .unwrap(),
        vec![Amount::new("1").unwrap(), Amount::new("2").unwrap()]
    );

    provider.set_response(
        &serde_json::to_string(&TradeSimulationResult {
            gas_used: Amount::new("21000").unwrap(),
            executed_buy_amount: Amount::new("1980").unwrap(),
            contract_balance: TradeSimulationBalanceDelta {
                sell_token_delta: SignedAmount::new("100").unwrap(),
                buy_token_delta: SignedAmount::new("-1980").unwrap(),
            },
            owner_balance: TradeSimulationBalanceDelta {
                sell_token_delta: SignedAmount::new("-100").unwrap(),
                buy_token_delta: SignedAmount::new("1980").unwrap(),
            },
        })
        .unwrap(),
    );

    let simulator = TradeSimulator {
        settlement_address: settlement_reader.settlement_address.clone(),
        settlement_abi_json: settlement_reader.settlement_abi_json.clone(),
        simulator_address: Address::new("0x6666666666666666666666666666666666666666").unwrap(),
        simulator_abi_json: serde_json::to_string(&["function simulateTrade(tuple,tuple[3])"])
            .unwrap(),
        provider: provider.clone(),
    };
    let trade = TradeSimulation {
        sell_token: Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(),
        buy_token: Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        receiver: None,
        sell_amount: Amount::new("100").unwrap(),
        buy_amount: Amount::new("200").unwrap(),
        sell_token_balance: None,
        buy_token_balance: Some(BuyTokenDestination::Internal),
        owner: Address::new("0x7777777777777777777777777777777777777777").unwrap(),
    };
    let result = simulator
        .simulate_trade(
            &trade,
            &[(
                InteractionStage::Pre,
                vec![cow_sdk_contracts::InteractionLike {
                    target: Address::new("0x8888888888888888888888888888888888888888").unwrap(),
                    value: None,
                    call_data: Some(Bytes::from_static(&[0x12, 0x34])),
                }],
            )],
        )
        .unwrap();
    assert_eq!(result.gas_used.to_string(), "21000");
    assert_eq!(result.executed_buy_amount.to_string(), "1980");

    let call = provider.calls.borrow().last().cloned().unwrap();
    assert_eq!(call.method, "simulateTrade");
    let payload: serde_json::Value = serde_json::from_str(&call.args_json).unwrap();
    let parameters = payload["parameters"].as_array().unwrap();
    assert_eq!(
        parameters[0]["receiver"],
        serde_json::json!("0x0000000000000000000000000000000000000000")
    );
    assert_eq!(
        parameters[0]["sellTokenBalance"],
        serde_json::json!(balance_id("erc20"))
    );
    assert_eq!(
        parameters[0]["buyTokenBalance"],
        serde_json::json!(balance_id("internal"))
    );
    assert_eq!(parameters[1][0][0]["callData"], serde_json::json!("0x1234"));
    assert_eq!(parameters[1][1], serde_json::json!([]));
    assert_eq!(parameters[1][2], serde_json::json!([]));
}
