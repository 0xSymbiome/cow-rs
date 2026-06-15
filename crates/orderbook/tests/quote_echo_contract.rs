//! Quote-echo binding contract (ADR 0058).
//!
//! A `/quote` response must echo every request-determined field; only the
//! variable price leg the solver returns is free. These tests pin the fixed-leg
//! fold for each side basis, every request-determined field check, the
//! deliberately-skipped cases, and the end-to-end fail-closed wiring through
//! [`OrderbookApi::quote`].

mod common;

use cow_sdk_core::{Address, AppDataHash, CowEnv};
use cow_sdk_orderbook::{
    Amount, BuyTokenDestination, OrderKind, OrderQuoteRequest, OrderQuoteResponse, OrderQuoteSide,
    OrderbookError, QuoteData, QuoteEchoField, SellTokenSource, SupportedChainId,
};
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use crate::common::{
    build_orderbook_api_with_base_url, default_context, sample_buy_token, sample_owner,
};

const SELL_TOKEN: &str = "0x1111111111111111111111111111111111111111";
const BUY_TOKEN: &str = "0x2222222222222222222222222222222222222222";
const FROM: &str = "0x3333333333333333333333333333333333333333";
const RECEIVER: &str = "0x4444444444444444444444444444444444444444";
const OTHER: &str = "0x5555555555555555555555555555555555555555";
const APP_DATA: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_HASH: &str = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn addr(hex: &str) -> Address {
    Address::new(hex).expect("test address literal must be valid")
}

fn amount(value: &str) -> Amount {
    Amount::new(value).expect("test amount literal must be valid")
}

fn app_data() -> AppDataHash {
    AppDataHash::new(APP_DATA).expect("test app-data hash literal must be valid")
}

/// A reconciling sell-before-fee pair: the request fixes a 1000 before-fee sell
/// and the response echoes it as `sellAmount 997 + feeAmount 3`, with every
/// request-determined field returned unchanged.
fn matching_sell() -> (OrderQuoteRequest, OrderQuoteResponse) {
    let request = OrderQuoteRequest::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        addr(FROM),
        OrderQuoteSide::sell(amount("1000")),
    )
    .with_receiver(addr(RECEIVER))
    .with_app_data_hash(app_data());

    let quote = QuoteData::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        amount("997"),
        amount("2000"),
        1_700_000_000,
        app_data(),
        OrderKind::Sell,
    )
    .with_network_cost_amount(amount("3"))
    .with_receiver(addr(RECEIVER));

    let response =
        OrderQuoteResponse::new(quote, "2026-01-01T00:00:00Z", true).with_from(addr(FROM));
    (request, response)
}

fn assert_mismatch(result: Result<(), OrderbookError>, expected: QuoteEchoField) {
    match result {
        Err(OrderbookError::QuoteEchoMismatch { field, .. }) => assert_eq!(
            field, expected,
            "wrong field discriminant on the echo mismatch"
        ),
        other => panic!("expected QuoteEchoMismatch({expected:?}), got {other:?}"),
    }
}

#[test]
fn honest_sell_before_fee_response_passes() {
    let (request, response) = matching_sell();
    response
        .ensure_matches(&request)
        .expect("an echoing response must pass");
}

#[test]
fn sell_after_fee_fold_passes_and_fails() {
    let request = OrderQuoteRequest::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        addr(FROM),
        OrderQuoteSide::sell_after_fee(amount("997")),
    );
    // The request pins no app-data, so an honest response echoes the zero hash.
    let quote = QuoteData::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        amount("997"),
        amount("2000"),
        1_700_000_000,
        AppDataHash::ZERO,
        OrderKind::Sell,
    )
    .with_network_cost_amount(amount("3"));
    let mut response = OrderQuoteResponse::new(quote, "2026-01-01T00:00:00Z", true);
    // After-fee: the sell amount passes through unchanged (no fee fold).
    response
        .ensure_matches(&request)
        .expect("an after-fee response that echoes the sell amount must pass");

    response.quote.sell_amount = amount("996");
    assert_mismatch(
        response.ensure_matches(&request),
        QuoteEchoField::FixedSellAmount,
    );
}

#[test]
fn buy_fold_passes_and_fails() {
    let request = OrderQuoteRequest::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        addr(FROM),
        OrderQuoteSide::buy(amount("2000")),
    );
    // The request pins no app-data, so an honest response echoes the zero hash.
    let quote = QuoteData::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        amount("997"),
        amount("2000"),
        1_700_000_000,
        AppDataHash::ZERO,
        OrderKind::Buy,
    );
    let mut response = OrderQuoteResponse::new(quote, "2026-01-01T00:00:00Z", true);
    response
        .ensure_matches(&request)
        .expect("a buy response that echoes the buy amount must pass");

    // The buy leg is fixed; shrinking it below the requested amount fails.
    response.quote.buy_amount = amount("1999");
    assert_mismatch(
        response.ensure_matches(&request),
        QuoteEchoField::FixedBuyAmount,
    );
}

#[test]
fn inflated_fixed_sell_leg_fails() {
    let (request, mut response) = matching_sell();
    response.quote.sell_amount = amount("5000");
    assert_mismatch(
        response.ensure_matches(&request),
        QuoteEchoField::FixedSellAmount,
    );
}

#[test]
fn inflated_fee_breaks_the_fold() {
    let (request, mut response) = matching_sell();
    response.quote = response.quote.with_network_cost_amount(amount("4"));
    assert_mismatch(
        response.ensure_matches(&request),
        QuoteEchoField::FixedSellAmount,
    );
}

#[test]
fn swapped_sell_token_fails() {
    let (request, mut response) = matching_sell();
    response.quote.sell_token = addr(OTHER);
    assert_mismatch(response.ensure_matches(&request), QuoteEchoField::SellToken);
}

#[test]
fn swapped_buy_token_fails() {
    let (request, mut response) = matching_sell();
    response.quote.buy_token = addr(OTHER);
    assert_mismatch(response.ensure_matches(&request), QuoteEchoField::BuyToken);
}

#[test]
fn flipped_kind_fails() {
    let (request, mut response) = matching_sell();
    response.quote.kind = OrderKind::Buy;
    assert_mismatch(response.ensure_matches(&request), QuoteEchoField::Kind);
}

#[test]
fn redirected_receiver_fails() {
    let (request, mut response) = matching_sell();
    response.quote.receiver = Some(addr(OTHER));
    assert_mismatch(response.ensure_matches(&request), QuoteEchoField::Receiver);
}

#[test]
fn flipped_balance_sources_fail() {
    let (request, mut response) = matching_sell();
    response.quote.sell_token_balance = SellTokenSource::External;
    assert_mismatch(
        response.ensure_matches(&request),
        QuoteEchoField::SellTokenBalance,
    );

    let (request, mut response) = matching_sell();
    response.quote.buy_token_balance = BuyTokenDestination::Internal;
    assert_mismatch(
        response.ensure_matches(&request),
        QuoteEchoField::BuyTokenBalance,
    );
}

#[test]
fn mismatched_owner_fails() {
    let (request, mut response) = matching_sell();
    response.from = Some(addr(OTHER));
    assert_mismatch(response.ensure_matches(&request), QuoteEchoField::From);
}

#[test]
fn swapped_pinned_app_data_hash_fails() {
    let (request, mut response) = matching_sell();
    response.quote.app_data = AppDataHash::new(OTHER_HASH).expect("valid hash");
    assert_mismatch(
        response.ensure_matches(&request),
        QuoteEchoField::AppDataHash,
    );
}

#[test]
fn stretched_absolute_valid_to_fails() {
    let request = OrderQuoteRequest::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        addr(FROM),
        OrderQuoteSide::sell(amount("1000")),
    )
    .with_valid_to(1_700_000_000);
    let quote = QuoteData::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        amount("1000"),
        amount("2000"),
        1_900_000_000,
        AppDataHash::ZERO,
        OrderKind::Sell,
    );
    let response = OrderQuoteResponse::new(quote, "2026-01-01T00:00:00Z", true);
    assert_mismatch(response.ensure_matches(&request), QuoteEchoField::ValidTo);
}

#[test]
fn relative_validity_does_not_check_valid_to() {
    // The request uses the default relative `validFor`, so a server-computed
    // `validTo` is not request-determined and must not be checked.
    let (request, mut response) = matching_sell();
    response.quote.valid_to = 1_999_999_999;
    response
        .ensure_matches(&request)
        .expect("a relative-validity request must not pin validTo");
}

#[test]
fn absent_owner_and_receiver_echoes_reconcile() {
    let request = OrderQuoteRequest::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        addr(FROM),
        OrderQuoteSide::sell(amount("1000")),
    );
    // No `from` echo on the response (an optional echo, skipped when absent) and
    // no receiver: an unset receiver reconciles to the owner on both sides, so
    // the response must pass rather than fail closed. The request pins no
    // app-data, so the honest echo is the zero hash.
    let quote = QuoteData::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        amount("997"),
        amount("2000"),
        1_700_000_000,
        AppDataHash::ZERO,
        OrderKind::Sell,
    )
    .with_network_cost_amount(amount("3"));
    let response = OrderQuoteResponse::new(quote, "2026-01-01T00:00:00Z", true);
    response
        .ensure_matches(&request)
        .expect("absent owner echo and an owner-equivalent receiver must reconcile");
}

#[test]
fn fabricated_receiver_echo_on_an_unpinned_request_fails() {
    // The request leaves the receiver unset, so the proceeds settle to the
    // owner. A response that fabricates a receiver redirects them; reconciling
    // the effective receiver fails closed even though the request pinned none.
    let request = OrderQuoteRequest::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        addr(FROM),
        OrderQuoteSide::sell(amount("1000")),
    );
    let quote = QuoteData::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        amount("997"),
        amount("2000"),
        1_700_000_000,
        AppDataHash::ZERO,
        OrderKind::Sell,
    )
    .with_network_cost_amount(amount("3"))
    .with_receiver(addr(OTHER));
    let response = OrderQuoteResponse::new(quote, "2026-01-01T00:00:00Z", true);
    assert_mismatch(response.ensure_matches(&request), QuoteEchoField::Receiver);
}

#[test]
fn owner_equivalent_receiver_echoes_pass() {
    // An unset request receiver settles to the owner, so a response that echoes
    // the owner explicitly, the zero sentinel, or nothing is owner-equivalent
    // and must reconcile rather than fail closed.
    let request = OrderQuoteRequest::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        addr(FROM),
        OrderQuoteSide::sell(amount("1000")),
    );
    for echoed in [Some(addr(FROM)), Some(Address::ZERO), None] {
        let quote = QuoteData::new(
            addr(SELL_TOKEN),
            addr(BUY_TOKEN),
            amount("997"),
            amount("2000"),
            1_700_000_000,
            AppDataHash::ZERO,
            OrderKind::Sell,
        )
        .with_network_cost_amount(amount("3"));
        let mut response = OrderQuoteResponse::new(quote, "2026-01-01T00:00:00Z", true);
        response.quote.receiver = echoed;
        response
            .ensure_matches(&request)
            .expect("owner-equivalent receiver echoes must reconcile");
    }
}

#[test]
fn full_document_request_binds_its_digest() {
    // A request that sends a full app-data document without pinning a hash binds
    // the document's keccak digest: the response must echo that digest, and a
    // substituted hash fails closed.
    const DOCUMENT: &str = r#"{"version":"1.1.0","metadata":{}}"#;
    let digest = AppDataHash::from_full_app_data(DOCUMENT);
    let request = OrderQuoteRequest::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        addr(FROM),
        OrderQuoteSide::sell(amount("1000")),
    )
    .with_app_data(DOCUMENT);
    let quote = QuoteData::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        amount("997"),
        amount("2000"),
        1_700_000_000,
        digest,
        OrderKind::Sell,
    )
    .with_network_cost_amount(amount("3"));
    let mut response = OrderQuoteResponse::new(quote, "2026-01-01T00:00:00Z", true);
    response
        .ensure_matches(&request)
        .expect("a response echoing the document digest must pass");

    response.quote.app_data = AppDataHash::new(OTHER_HASH).expect("valid hash");
    assert_mismatch(
        response.ensure_matches(&request),
        QuoteEchoField::AppDataHash,
    );
}

#[test]
fn omitted_app_data_must_echo_the_zero_hash() {
    // A request that pins no app-data must see the zero hash echoed back; a
    // non-zero hash on an omitted pair is a server-fabricated commitment.
    let request = OrderQuoteRequest::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        addr(FROM),
        OrderQuoteSide::sell(amount("1000")),
    );
    let quote = QuoteData::new(
        addr(SELL_TOKEN),
        addr(BUY_TOKEN),
        amount("997"),
        amount("2000"),
        1_700_000_000,
        AppDataHash::ZERO,
        OrderKind::Sell,
    )
    .with_network_cost_amount(amount("3"));
    let mut response = OrderQuoteResponse::new(quote, "2026-01-01T00:00:00Z", true);
    response
        .ensure_matches(&request)
        .expect("the zero-hash echo for an omitted pair must pass");

    response.quote.app_data = app_data();
    assert_mismatch(
        response.ensure_matches(&request),
        QuoteEchoField::AppDataHash,
    );
}

#[tokio::test]
async fn quote_fails_closed_end_to_end_on_a_tampered_fixed_leg() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/quote"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "quote": {
                "sellToken": sample_owner().to_hex_string(),
                "buyToken": sample_buy_token().to_hex_string(),
                "sellAmount": "1000",
                "buyAmount": "900",
                "validTo": 1_700_000_000,
                "appData": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "feeAmount": "10",
                "kind": "sell",
                "partiallyFillable": false,
                "sellTokenBalance": "erc20",
                "buyTokenBalance": "erc20"
            },
            "from": sample_owner().to_hex_string(),
            "expiration": "2026-01-01T00:00:00Z",
            "id": 1,
            "verified": true
        })))
        .mount(&server)
        .await;

    let api = build_orderbook_api_with_base_url(
        default_context(SupportedChainId::GnosisChain, CowEnv::Prod),
        server.uri(),
    );

    // The response echoes a fixed leg of 1000 + 10 = 1010, but the request asks
    // for a before-fee 2000, so the gate must fail closed at the real API
    // boundary before any signable order is produced.
    let error = api
        .quote(&OrderQuoteRequest::new(
            sample_owner(),
            sample_buy_token(),
            sample_owner(),
            OrderQuoteSide::sell(amount("2000")),
        ))
        .await
        .expect_err("a tampered fixed leg must fail closed through OrderbookApi::quote");
    assert_mismatch(Err(error), QuoteEchoField::FixedSellAmount);
}
