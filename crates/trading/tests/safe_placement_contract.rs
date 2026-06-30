#![allow(
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::too_many_lines,
    reason = "small integration-test helpers do not need public API lint polish"
)]

//! Authorization-as-a-value placement (ADR 0073): the typed [`OrderPlacement`]
//! sum, the bundled pre-sign activation, and the honest EIP-1271 / pre-sign
//! lifecycle helpers, exercised against the deterministic mock harness.

mod common;

use cow_sdk_contracts::{
    ContractId, approve_transaction, pre_sign_transaction, resolve_contract_address,
};
use cow_sdk_core::{Amount, CowEnv, HexData, OrderKind, SupportedChainId};
use cow_sdk_orderbook::{OrderStatus, SigningScheme};
use cow_sdk_trading::{
    Authorization, OrderPlacement, build_limit_order_to_sign, build_presign_activation,
    place_limit, place_swap, post_limit_order_presign, post_swap_order_presign, preflight_eip1271,
    presign_activation_status, quote_results,
};

use crate::common::{
    MockEip1271Provider, MockOrderbook, MockProvider, MockSigner, OWNER, address, order_uid,
    regular_order, sample_limit_parameters, sample_trade_parameters, sample_trader_parameters,
    sell_quote_response,
};

const SEPOLIA: SupportedChainId = SupportedChainId::Sepolia;

fn new_orderbook() -> MockOrderbook {
    MockOrderbook::new(SEPOLIA, sell_quote_response())
}

#[tokio::test]
async fn place_limit_presign_returns_pending_activation_with_ordered_calls() {
    let trader = sample_trader_parameters();
    let orderbook = new_orderbook();
    let params = sample_limit_parameters(OrderKind::Sell);
    let sell_token = params.sell_token;
    let sell_amount = params.sell_amount;

    let placement = place_limit(
        params,
        address(OWNER),
        Authorization::pre_sign(),
        &trader,
        None,
        &orderbook,
    )
    .await
    .expect("pre-sign limit placement should succeed");

    // The scheme statically selects the pending-activation arm.
    let OrderPlacement::PendingActivation {
        order_uid: placed_uid,
        activation,
    } = placement
    else {
        panic!("pre-sign placement must resolve to PendingActivation");
    };
    assert_eq!(placed_uid, order_uid());

    // The posted order carries the pre-sign scheme and the owner-address wire
    // signature the existing dispatch emits (both that and empty 0x are
    // orderbook-accepted; the working path is preserved).
    let posted = orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .expect("a pre-sign order must be recorded");
    assert_eq!(posted.signing_scheme, SigningScheme::PreSign);

    // The activation is the ordered approve-then-set-pre-signature pair, each a
    // gas-free zero-value call built by the existing contract encoders.
    assert_eq!(activation.calls.len(), 2);
    let vault_relayer =
        resolve_contract_address(ContractId::VaultRelayer, None, SEPOLIA, CowEnv::Prod)
            .expect("vault relayer is registered on sepolia");
    let expected_approve = approve_transaction(sell_token, vault_relayer, sell_amount);
    let expected_set_pre_signature = pre_sign_transaction(
        &order_uid(),
        SEPOLIA,
        Some(&cow_sdk_core::ProtocolOptions::new().with_env(CowEnv::Prod)),
    )
    .expect("set-pre-signature tx should build");

    assert_eq!(activation.calls[0], expected_approve);
    assert_eq!(activation.calls[0].value, Amount::ZERO);
    assert_eq!(activation.calls[1], expected_set_pre_signature);
    assert_eq!(activation.calls[1].value, Amount::ZERO);
}

#[tokio::test]
async fn place_limit_eip1271_is_live_with_provider_blob_and_scheme() {
    let trader = sample_trader_parameters();
    let orderbook = new_orderbook();
    let params = sample_limit_parameters(OrderKind::Sell);

    let placement = place_limit(
        params,
        address(OWNER),
        Authorization::eip1271(std::sync::Arc::new(MockEip1271Provider)),
        &trader,
        None,
        &orderbook,
    )
    .await
    .expect("eip1271 limit placement should succeed");

    // An EIP-1271 order is valid once posted.
    let OrderPlacement::Live { order_uid: placed } = placement else {
        panic!("eip1271 placement must resolve to Live");
    };
    assert_eq!(placed, order_uid());

    let posted = orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .expect("an eip1271 order must be recorded");
    assert_eq!(posted.signing_scheme, SigningScheme::Eip1271);
    assert_eq!(posted.signature, "0x7e57c0de");
}

#[tokio::test]
async fn place_limit_eip1271_posts_the_boundary_builder_order() {
    // This is the native shape of the wasm `placeLimit` eip1271 arm: the boundary
    // builder produces the unsigned order, a contract signature is resolved
    // against it, and `place_limit` posts the order under the eip1271 scheme. The
    // posted order must be the boundary order field-for-field, proving the
    // resolved signature is over the digest the order carries on the wire.
    let trader = sample_trader_parameters();
    let orderbook = new_orderbook();
    // A pinned `valid_to` keeps `order_to_sign` deterministic so the boundary
    // build and the placement build cannot differ by a clock tick.
    let params = sample_limit_parameters(OrderKind::Sell).with_valid_to(2_000_000_000);

    let (boundary_order, boundary_app_data) =
        build_limit_order_to_sign(&params, address(OWNER), &trader, None)
            .await
            .expect("boundary builder should succeed");

    let placement = place_limit(
        params,
        address(OWNER),
        Authorization::eip1271(std::sync::Arc::new(MockEip1271Provider)),
        &trader,
        None,
        &orderbook,
    )
    .await
    .expect("eip1271 limit placement should succeed");

    let OrderPlacement::Live { order_uid: placed } = placement else {
        panic!("eip1271 placement must resolve to Live");
    };
    assert_eq!(placed, order_uid());

    let posted = orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .expect("an eip1271 order must be recorded");

    // The posted order is the boundary order: same amounts, validity, receiver,
    // app-data hash, and balance sources. If the boundary builder forked from the
    // posting path, these would diverge and the wallet would have signed a digest
    // the order does not carry.
    assert_eq!(posted.sell_token, boundary_order.sell_token);
    assert_eq!(posted.buy_token, boundary_order.buy_token);
    assert_eq!(posted.sell_amount, boundary_order.sell_amount);
    assert_eq!(posted.buy_amount, boundary_order.buy_amount);
    assert_eq!(posted.valid_to, boundary_order.valid_to);
    assert_eq!(posted.receiver, Some(boundary_order.receiver));
    assert_eq!(posted.app_data_hash, Some(boundary_order.app_data));
    assert_eq!(
        posted.app_data_hash,
        Some(boundary_app_data.app_data_keccak256)
    );
    assert_eq!(posted.kind, boundary_order.kind);
    assert_eq!(posted.partially_fillable, boundary_order.partially_fillable);
    assert_eq!(posted.sell_token_balance, boundary_order.sell_token_balance);
    assert_eq!(posted.buy_token_balance, boundary_order.buy_token_balance);

    // The order is live under the eip1271 scheme carrying the resolved blob.
    assert_eq!(posted.signing_scheme, SigningScheme::Eip1271);
    assert_eq!(posted.signature, "0x7e57c0de");
}

#[tokio::test]
async fn place_swap_presign_matches_standalone_activation() {
    let trader = sample_trader_parameters();
    let signer = MockSigner::default();
    let orderbook = new_orderbook();
    let trade = sample_trade_parameters(OrderKind::Sell);

    let quote = quote_results(&trade, &trader, &signer, None, &orderbook)
        .await
        .expect("quote should succeed");

    let placement = place_swap(
        &quote,
        address(OWNER),
        Authorization::pre_sign(),
        &trader,
        None,
        &orderbook,
    )
    .await
    .expect("pre-sign swap placement should succeed");
    let OrderPlacement::PendingActivation {
        order_uid: placed,
        activation,
    } = placement
    else {
        panic!("pre-sign swap placement must resolve to PendingActivation");
    };
    assert_eq!(placed, order_uid());

    // The bundled activation equals the standalone builder for the same UID,
    // sell token, and signed sell amount.
    let standalone = build_presign_activation(
        &order_uid(),
        quote.trade_parameters.sell_token,
        quote.order_to_sign.sell_amount,
        SEPOLIA,
        Some(&cow_sdk_core::ProtocolOptions::new().with_env(CowEnv::Prod)),
    )
    .expect("standalone activation should build");
    assert_eq!(activation.calls, standalone.calls);
}

#[tokio::test]
async fn build_presign_activation_is_pure_and_targets_relayer_then_settlement() {
    let sell_token = address(common::WETH);
    let amount = Amount::new("1000000000000000000").expect("amount literal must be valid");

    let activation = build_presign_activation(
        &order_uid(),
        sell_token,
        amount,
        SEPOLIA,
        Some(&cow_sdk_core::ProtocolOptions::new().with_env(CowEnv::Prod)),
    )
    .expect("activation should build");

    let vault_relayer =
        resolve_contract_address(ContractId::VaultRelayer, None, SEPOLIA, CowEnv::Prod)
            .expect("vault relayer is registered on sepolia");
    let settlement = resolve_contract_address(ContractId::Settlement, None, SEPOLIA, CowEnv::Prod)
        .expect("settlement is registered on sepolia");

    assert_eq!(activation.calls[0].to, sell_token);
    assert_eq!(activation.calls[1].to, settlement);
    assert_eq!(
        activation.calls[0],
        approve_transaction(sell_token, vault_relayer, amount)
    );
}

#[tokio::test]
async fn preflight_eip1271_returns_real_verdict() {
    let provider = MockProvider::default();
    let owner = address(OWNER);
    let digest = cow_sdk_core::Hash32::from_bytes([7u8; 32]);
    provider.set_code(&owner, "0x1234");
    provider.set_contract_response("isValidSignature", "\"0x1626ba7e\"");

    preflight_eip1271(
        &provider,
        owner,
        digest,
        HexData::new("0xdeadbeef").unwrap(),
    )
    .await
    .expect("a magic-value response is a valid verdict");

    // A non-magic response surfaces a real rejection, not a silent "valid".
    provider.set_contract_response("isValidSignature", "\"0xffffffff\"");
    let rejected = preflight_eip1271(
        &provider,
        owner,
        digest,
        HexData::new("0xdeadbeef").unwrap(),
    )
    .await;
    assert!(rejected.is_err(), "a non-magic response must reject");
}

#[test]
fn presign_activation_status_maps_pending_to_open_and_passes_terminal() {
    let mut order = regular_order();

    order.status = OrderStatus::PresignaturePending;
    assert_eq!(presign_activation_status(&order), OrderStatus::Open);

    order.status = OrderStatus::Open;
    assert_eq!(presign_activation_status(&order), OrderStatus::Open);

    order.status = OrderStatus::Expired;
    assert_eq!(presign_activation_status(&order), OrderStatus::Expired);

    order.status = OrderStatus::Fulfilled;
    assert_eq!(presign_activation_status(&order), OrderStatus::Fulfilled);
}

#[tokio::test]
async fn free_function_presign_entries_stay_available() {
    // The new typed-sum surface is additive: the underlying signer-less pre-sign
    // posting entries the placement path reuses remain callable directly.
    let trader = sample_trader_parameters();
    let signer = MockSigner::default();
    let orderbook = new_orderbook();
    let params = sample_limit_parameters(OrderKind::Sell);

    let limit = post_limit_order_presign(&params, &trader, None, &orderbook)
        .await
        .expect("limit pre-sign entry should remain available");
    assert_eq!(limit.signing_scheme, SigningScheme::PreSign);

    let trade = sample_trade_parameters(OrderKind::Sell);
    let quote = quote_results(&trade, &trader, &signer, None, &orderbook)
        .await
        .expect("quote should succeed");
    let swap = post_swap_order_presign(&quote, &trader, None, &orderbook)
        .await
        .expect("swap pre-sign entry should be available");
    assert_eq!(swap.signing_scheme, SigningScheme::PreSign);
}
