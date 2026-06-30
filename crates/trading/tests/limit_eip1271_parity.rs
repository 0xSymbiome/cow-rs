#![cfg(not(target_arch = "wasm32"))]
#![allow(
    clippy::missing_const_for_fn,
    clippy::too_many_lines,
    reason = "integration-test helpers do not need public API lint polish"
)]

//! Digest-parity guard for the limit EIP-1271 boundary builder.
//!
//! [`build_limit_order_to_sign`] exposes the unsigned limit order at a boundary
//! so a smart-account contract signature can be resolved against it before the
//! order enters the native placement path. Correctness depends on that boundary
//! order being byte-identical to the order [`post_limit_order`] signs for the
//! same inputs: the resolved signature is over the boundary digest, while the
//! posted order carries the digest the posting path rebuilds. The builder shares
//! its limit-defaults and `order_to_sign` steps with the posting path, so the two
//! agree by construction; these tests fail CI if a future change forks one path
//! from the other (slippage default, `apply_costs_slippage_and_fees`, the app-data
//! class, or the `order_to_sign` parameters).

mod common;

use cow_sdk_core::OrderKind;
use cow_sdk_orderbook::SigningScheme;
use cow_sdk_trading::{
    LimitTradeParams, QuoteRequestOverride, SupportedChainId, TradeAdvancedSettings,
    build_limit_order_to_sign, post_limit_order,
};

use crate::common::{
    MockOrderbook, MockSigner, OWNER, address, sample_limit_parameters, sample_trader_parameters,
    sell_quote_response,
};

const SEPOLIA: SupportedChainId = SupportedChainId::Sepolia;

fn new_orderbook() -> MockOrderbook {
    MockOrderbook::new(SEPOLIA, sell_quote_response())
}

/// A pinned `valid_to` keeps `order_to_sign` deterministic across the two builds:
/// without it `order_to_sign` defaults `valid_to` from `SystemTime::now()`, which
/// could differ by a second between the boundary build and the posting build and
/// produce a spurious digest mismatch unrelated to the code paths under test.
fn pinned_params(kind: OrderKind) -> LimitTradeParams {
    sample_limit_parameters(kind).with_valid_to(2_000_000_000)
}

/// Drives the native posting path far enough to capture the `order_to_sign` it
/// builds for the given inputs (the mock orderbook records the posted body and
/// the result echoes the unsigned order).
async fn native_post_order_to_sign(
    params: &LimitTradeParams,
    advanced: Option<&TradeAdvancedSettings>,
) -> cow_sdk_core::OrderData {
    let trader = sample_trader_parameters();
    let signer = MockSigner::default();
    let orderbook = new_orderbook();
    post_limit_order(params, &trader, &signer, advanced, &orderbook)
        .await
        .expect("native limit post should succeed")
        .order_to_sign
}

#[tokio::test]
async fn boundary_builder_digest_equals_native_post_digest_sell() {
    let trader = sample_trader_parameters();
    let params = pinned_params(OrderKind::Sell);

    let (boundary_order, boundary_app_data) =
        build_limit_order_to_sign(&params, address(OWNER), &trader, None)
            .await
            .expect("boundary builder should succeed");

    let native_order = native_post_order_to_sign(&params, None).await;

    assert_eq!(
        boundary_order, native_order,
        "boundary digest must equal native post digest"
    );
    // The app-data hash the boundary builder returns is the same one the posted
    // order binds (it is the `app_data` field the posting path signs).
    assert_eq!(
        boundary_app_data.app_data_keccak256, native_order.app_data,
        "boundary app-data hash must equal the posted order's app-data hash"
    );
}

#[tokio::test]
async fn boundary_builder_digest_equals_native_post_digest_buy() {
    let trader = sample_trader_parameters();
    let params = pinned_params(OrderKind::Buy);

    let (boundary_order, _boundary_app_data) =
        build_limit_order_to_sign(&params, address(OWNER), &trader, None)
            .await
            .expect("boundary builder should succeed");

    let native_order = native_post_order_to_sign(&params, None).await;

    assert_eq!(
        boundary_order, native_order,
        "boundary digest must equal native post digest for a buy order"
    );
}

#[tokio::test]
async fn boundary_builder_digest_equals_native_post_digest_with_advanced_settings() {
    // Advanced settings exercise the full defaults pipeline both paths share:
    // a quote-request override (receiver / validity) and an EIP-1271 signing
    // scheme that the boundary builder ignores for the digest but the posting
    // path threads through. The digest must still match.
    let trader = sample_trader_parameters();
    let params = pinned_params(OrderKind::Sell);
    let advanced = TradeAdvancedSettings::new().with_quote_request(
        QuoteRequestOverride::new()
            .with_from(address(OWNER))
            .with_signing_scheme(SigningScheme::Eip1271),
    );

    let (boundary_order, _boundary_app_data) =
        build_limit_order_to_sign(&params, address(OWNER), &trader, Some(&advanced))
            .await
            .expect("boundary builder should succeed");

    let native_order = native_post_order_to_sign(&params, Some(&advanced)).await;

    assert_eq!(
        boundary_order, native_order,
        "boundary digest must equal native post digest under advanced settings"
    );
}

#[tokio::test]
async fn boundary_builder_injects_owner_as_from_and_receiver_default() {
    // The builder injects `owner` so `from == owner`; with no explicit receiver
    // the order's receiver defaults to the owner, exactly as the posting path
    // resolves it. A params value with no owner set still yields an owner-bound
    // order, matching how the placement path overrides the DTO owner.
    let trader = sample_trader_parameters();
    let mut params = pinned_params(OrderKind::Sell);
    params.owner = None;
    params.receiver = None;

    let (boundary_order, _app_data) =
        build_limit_order_to_sign(&params, address(OWNER), &trader, None)
            .await
            .expect("boundary builder should succeed");

    assert_eq!(boundary_order.receiver, address(OWNER));

    // The native post path, given the same owner via params, resolves the same
    // receiver default.
    let mut native_params = params.clone();
    native_params.owner = Some(address(OWNER));
    let native_order = native_post_order_to_sign(&native_params, None).await;
    assert_eq!(boundary_order, native_order);
}
