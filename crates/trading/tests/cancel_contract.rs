mod common;

use cow_sdk_core::SupportedChainId;
use cow_sdk_trading::{OrderTraderParameters, off_chain_cancel_order};

use crate::common::{
    MockOrderbook, MockSigner, order_uid, sample_trader_parameters, sell_quote_response,
};

#[tokio::test]
async fn offchain_cancellation_signs_and_dispatches_order_uids_to_orderbook() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let signer = MockSigner::default();
    let params = OrderTraderParameters {
        order_uid: order_uid(),
        chain_id: Some(trader.chain_id),
        env: trader.env,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
    };

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
    let params = OrderTraderParameters {
        order_uid: order_uid(),
        chain_id: Some(SupportedChainId::Mainnet),
        env: trader.env,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
    };

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
