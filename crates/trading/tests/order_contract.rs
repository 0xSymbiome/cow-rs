mod common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use cow_sdk_core::{
    Amount, CowEnv, MAX_VALID_TO_EPOCH, OrderBalance, ProtocolOptions, SupportedChainId,
    UnsignedOrder, eth_flow_contract_address, wrapped_native_token,
};
use cow_sdk_signing::generate_order_id;
use cow_sdk_trading::{
    OrderToSignParams, TradingError, calculate_unique_order_id, get_order_to_sign,
};
use tokio::time::timeout;

use crate::common::{MockEthFlowChecker, OWNER, address, app_data_hash, sample_limit_parameters};

fn sample_ethflow_order(buy_amount: &str) -> UnsignedOrder {
    UnsignedOrder {
        sell_token: address("0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"),
        buy_token: address("0x0625aFB445C3B6B7B929342a04A22599fd5dBB59"),
        receiver: address("0xc8c753Ee51E8Fc80e199AB297fB575634a1aC1d3"),
        sell_amount: Amount::new("1000000000000000000")
            .expect("test sell amount literal must be valid"),
        buy_amount: Amount::new(buy_amount).expect("test buy amount literal must be valid"),
        valid_to: 1_700_000_000,
        app_data: app_data_hash(),
        fee_amount: Amount::zero(),
        kind: cow_sdk_core::OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: OrderBalance::Erc20,
        buy_token_balance: OrderBalance::Erc20,
    }
}

#[tokio::test]
async fn unique_order_id_decrements_buy_amount_after_a_collision() {
    let chain_id = SupportedChainId::Sepolia;
    let order = sample_ethflow_order("500");
    let checker = MockEthFlowChecker {
        results: Arc::new(Mutex::new(vec![true, false])),
    };
    let options = ProtocolOptions::new().with_env(CowEnv::Prod);

    let generated = calculate_unique_order_id(chain_id, &order, Some(&checker), Some(&options))
        .await
        .expect("collision retry should produce a unique order id");

    let mut expected_order = order.clone();
    expected_order.valid_to = MAX_VALID_TO_EPOCH;
    expected_order.sell_token = wrapped_native_token(chain_id).address;
    expected_order.buy_amount =
        Amount::new("499").expect("decremented buy amount literal must remain valid");
    let expected_owner = eth_flow_contract_address(chain_id, CowEnv::Prod);
    let expected = generate_order_id(chain_id, &expected_order, &expected_owner, Some(&options))
        .expect("expected order id generation must succeed");

    assert_eq!(generated.order_id, expected.order_id);
    assert_eq!(generated.order_digest, expected.order_digest);
}

#[tokio::test]
async fn unique_order_id_returns_immediately_when_no_collision_exists() {
    let chain_id = SupportedChainId::Sepolia;
    let order = sample_ethflow_order("500");

    let generated = timeout(
        Duration::from_millis(250),
        calculate_unique_order_id(chain_id, &order, None, None),
    )
    .await
    .expect("the no-collision path must not loop after the first generated id")
    .expect("the first generated order id must remain valid");

    let mut expected_order = order.clone();
    expected_order.valid_to = MAX_VALID_TO_EPOCH;
    expected_order.sell_token = wrapped_native_token(chain_id).address;
    let expected_owner = eth_flow_contract_address(chain_id, CowEnv::Prod);
    let expected = generate_order_id(chain_id, &expected_order, &expected_owner, None)
        .expect("expected order id generation must succeed");

    assert_eq!(generated.order_id, expected.order_id);
    assert_eq!(generated.order_digest, expected.order_digest);
}

#[tokio::test]
async fn unique_order_id_keeps_the_first_generated_value_when_checker_reports_no_collision() {
    let chain_id = SupportedChainId::Sepolia;
    let order = sample_ethflow_order("1");
    let checker = MockEthFlowChecker {
        results: Arc::new(Mutex::new(vec![false])),
    };

    let generated = calculate_unique_order_id(chain_id, &order, Some(&checker), None)
        .await
        .expect("a non-collision result must return the first generated id");

    let mut expected_order = order.clone();
    expected_order.valid_to = MAX_VALID_TO_EPOCH;
    expected_order.sell_token = wrapped_native_token(chain_id).address;
    let expected_owner = eth_flow_contract_address(chain_id, CowEnv::Prod);
    let expected = generate_order_id(chain_id, &expected_order, &expected_owner, None)
        .expect("expected order id generation must succeed");

    assert_eq!(generated.order_id, expected.order_id);
    assert_eq!(generated.order_digest, expected.order_digest);
}

#[tokio::test]
async fn unique_order_id_rejects_zero_buy_amount_when_a_collision_requires_a_retry() {
    let chain_id = SupportedChainId::Sepolia;
    let order = sample_ethflow_order("0");
    let checker = MockEthFlowChecker {
        results: Arc::new(Mutex::new(vec![true])),
    };

    let error = calculate_unique_order_id(chain_id, &order, Some(&checker), None)
        .await
        .expect_err("zero buy amount must fail when a retry is required");

    assert!(matches!(
        error,
        TradingError::InvalidInput(message) if message == "buyAmount must be greater than 0: 0"
    ));
}

#[test]
fn get_order_to_sign_preserves_non_default_balance_semantics() {
    let mut params = sample_limit_parameters(cow_sdk_core::OrderKind::Sell);
    params.sell_token_balance = OrderBalance::External;
    params.buy_token_balance = OrderBalance::Internal;

    let order = get_order_to_sign(
        OrderToSignParams::new(SupportedChainId::Sepolia, address(OWNER), false)
            .with_apply_costs_slippage_and_fees(false),
        &params,
        &app_data_hash(),
    )
    .expect("order construction should preserve configured balances");

    assert_eq!(order.sell_token_balance, OrderBalance::External);
    assert_eq!(order.buy_token_balance, OrderBalance::Internal);
}
