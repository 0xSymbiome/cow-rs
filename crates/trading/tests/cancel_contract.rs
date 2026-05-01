mod common;

use cow_sdk_core::{Amount, SupportedChainId};
use cow_sdk_trading::{
    GAS_LIMIT_DEFAULT, OrderTraderParameters, off_chain_cancel_order,
    onchain_cancellation_transaction,
};

use crate::common::{
    MockOrderbook, MockSigner, order_uid, sample_trader_parameters, sell_quote_response,
};

#[tokio::test]
async fn offchain_cancellation_signs_and_dispatches_order_uids_to_orderbook() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let signer = MockSigner::default();
    let mut params = OrderTraderParameters::new(order_uid()).with_chain_id(trader.chain_id);
    if let Some(env) = trader.env {
        params = params.with_env(env);
    }

    let cancelled = off_chain_cancel_order(&orderbook, &params, &trader, &signer)
        .await
        .expect("off-chain cancellation should succeed");
    let state = orderbook.state();
    let cancellation = state
        .cancellations
        .last()
        .cloned()
        .expect("cancellation payload must be sent");

    assert!(cancelled);
    assert_eq!(cancellation.order_uids, vec![order_uid()]);
    assert!(!cancellation.signature.is_empty());
    assert_eq!(
        cancellation.signing_scheme,
        cow_sdk_orderbook::EcdsaSigningScheme::Eip712
    );
}

#[tokio::test]
async fn offchain_cancellation_rejects_call_level_chain_conflicts_with_orderbook_context() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let signer = MockSigner::default();
    let mut params =
        OrderTraderParameters::new(order_uid()).with_chain_id(SupportedChainId::Mainnet);
    if let Some(env) = trader.env {
        params = params.with_env(env);
    }

    let error = off_chain_cancel_order(&orderbook, &params, &trader, &signer)
        .await
        .expect_err("mismatched cancellation chain must fail before signing");

    assert!(matches!(
        error,
        cow_sdk_trading::TradingError::InjectedOrderbookContextConflict {
            field: "chainId",
            ..
        }
    ));
    assert!(orderbook.state().cancellations.is_empty());
}

#[test]
fn cancellation_gas_estimation_fallback_uses_documented_constant() {
    let signer = MockSigner::default();
    signer
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .estimated_gas = Err("gas estimation unavailable".to_owned());

    let tx = onchain_cancellation_transaction(
        &signer,
        SupportedChainId::Sepolia,
        &crate::common::regular_order(),
        None,
    )
    .expect("on-chain cancellation must fall back when gas estimation fails");

    assert_eq!(
        tx.gas_limit,
        Some(Amount::new(GAS_LIMIT_DEFAULT.to_string()).expect("default gas literal is valid")),
    );
}
