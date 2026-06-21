#![cfg(target_arch = "wasm32")]

//! Parity between the annotation-only wasm response DTOs and the native
//! `cow_sdk_orderbook` serde shapes they mirror.
//!
//! `getOrder` / `getOrders` / `getTrades` / `getQuote` serialize the **native**
//! orderbook types directly across the ABI; the matching `…Dto` structs
//! (`crates/wasm/src/exports/dto/order.rs`, `…/orderbook.rs`) exist only to
//! generate the TypeScript declarations. Because the binding never constructs a
//! DTO from a native value, the compiler cannot catch divergence between the
//! runtime JSON (native serde) and the declared shape (DTO). The `.d.ts`
//! snapshot guards `DTO → declaration`, not `native → DTO`.
//!
//! These tests close that gap: they round-trip the **same upstream golden
//! fixtures** the native types are validated against in
//! `crates/orderbook/tests/wire_contract.rs` through the DTO mirrors, so
//! a field rename, retype, or drop on either side fails closed. The fixtures are
//! embedded with `include_str!` because `wasm32` has no filesystem at test time.
//! `QuoteDataDto` is exercised transitively through `OrderQuoteResponseDto`,
//! whose `quote` field carries it.

use wasm_bindgen_test::*;

use cow_sdk_wasm::exports::{
    CompetitionOrderStatusDto, NativePriceResponseDto, OrderDto, OrderQuoteResponseDto,
    SolverCompetitionResponseDto, TotalSurplusDto, TradeDto,
};

wasm_bindgen_test_configure!(run_in_browser);

const ORDER_FIXTURE: &str =
    include_str!("../../../parity/fixtures/orderbook/order_with_full_metadata.json");
const ORDER_QUOTE_RESPONSE_FIXTURE: &str =
    include_str!("../../../parity/fixtures/orderbook/order_quote_response.json");
const TRADE_FIXTURE: &str = include_str!("../../../parity/fixtures/orderbook/trade.json");
const TOTAL_SURPLUS_FIXTURE: &str =
    include_str!("../../../parity/fixtures/orderbook/total_surplus.json");
const SOLVER_COMPETITION_FIXTURE: &str =
    include_str!("../../../parity/fixtures/orderbook/solver_competition_response.json");

/// Asserts every field present in `fixture` survives a `fixture → DTO → JSON`
/// round-trip with an identical value — the same field-preservation contract
/// `wire_contract.rs::assert_fixture_roundtrips_field_for_field` enforces on the
/// native type,
/// applied here to the DTO mirror. SDK-computed fields the DTO adds on output
/// (for example `totalFee`) are permitted; a dropped or changed *fixture* field
/// is not.
fn assert_fixture_preserved(label: &str, fixture: &str, rendered: &serde_json::Value) {
    let expected = fixture_payload(fixture);
    let expected_obj = expected
        .as_object()
        .expect("fixture payload must be a JSON object");
    let actual_obj = rendered
        .as_object()
        .expect("serialized DTO root must be a JSON object");
    for (field, value) in expected_obj {
        assert_eq!(
            actual_obj.get(field),
            Some(value),
            "{label}: serialized DTO dropped or changed fixture field `{field}`",
        );
    }
}

/// Extracts the wire document from a fixture's `payload` envelope; the
/// provenance header around it is validated by `cargo parity-validate`.
fn fixture_payload(fixture: &str) -> serde_json::Value {
    let parsed: serde_json::Value =
        serde_json::from_str(fixture).expect("fixture must be valid JSON");
    parsed["payload"].clone()
}

#[wasm_bindgen_test]
fn order_dto_mirrors_native_order_fixture() {
    let dto: OrderDto = serde_json::from_value(fixture_payload(ORDER_FIXTURE))
        .expect("OrderDto must deserialize the native fixture");
    let rendered = serde_json::to_value(&dto).expect("OrderDto must serialize");
    assert_fixture_preserved("OrderDto", ORDER_FIXTURE, &rendered);
}

#[wasm_bindgen_test]
fn order_quote_response_dto_mirrors_native_fixture() {
    // Round-tripping the response also exercises the nested `QuoteDataDto`
    // through the `quote` field: a drift in `QuoteDataDto` changes the rendered
    // `quote` object and fails the top-level field comparison.
    let dto: OrderQuoteResponseDto =
        serde_json::from_value(fixture_payload(ORDER_QUOTE_RESPONSE_FIXTURE))
            .expect("OrderQuoteResponseDto must deserialize the native fixture");
    let rendered = serde_json::to_value(&dto).expect("OrderQuoteResponseDto must serialize");
    assert_fixture_preserved(
        "OrderQuoteResponseDto",
        ORDER_QUOTE_RESPONSE_FIXTURE,
        &rendered,
    );
}

#[wasm_bindgen_test]
fn trade_dto_mirrors_native_trade_fixture() {
    let dto: TradeDto = serde_json::from_value(fixture_payload(TRADE_FIXTURE))
        .expect("TradeDto must deserialize the native fixture");
    let rendered = serde_json::to_value(&dto).expect("TradeDto must serialize");
    assert_fixture_preserved("TradeDto", TRADE_FIXTURE, &rendered);
}

#[wasm_bindgen_test]
fn native_price_response_dto_mirrors_native_shape() {
    // `NativePriceResponse` carries a single `price` field and has no golden
    // fixture; pin the wire shape inline (under the same payload envelope the
    // committed fixtures use) so a rename or retype still fails.
    const FIXTURE: &str = r#"{"payload":{"price":1234.5}}"#;
    let dto: NativePriceResponseDto = serde_json::from_value(fixture_payload(FIXTURE))
        .expect("NativePriceResponseDto must deserialize");
    let rendered = serde_json::to_value(&dto).expect("NativePriceResponseDto must serialize");
    assert_fixture_preserved("NativePriceResponseDto", FIXTURE, &rendered);
}

#[wasm_bindgen_test]
fn total_surplus_dto_mirrors_native_fixture() {
    let dto: TotalSurplusDto = serde_json::from_value(fixture_payload(TOTAL_SURPLUS_FIXTURE))
        .expect("TotalSurplusDto must deserialize the native fixture");
    let rendered = serde_json::to_value(&dto).expect("TotalSurplusDto must serialize");
    assert_fixture_preserved("TotalSurplusDto", TOTAL_SURPLUS_FIXTURE, &rendered);
}

#[wasm_bindgen_test]
fn solver_competition_response_dto_mirrors_native_fixture() {
    // `SolverCompetitionResponseDto` is the highest-field-count shadow DTO (the
    // CIP-67 `/solver_competition/*` contract): it nests `CompetitionAuctionDto`
    // and a `SolverSettlementDto` vector that the binding never constructs from a
    // native value, so only this fixture round-trip catches a rename, retype, or
    // drop between `cow_sdk_orderbook::SolverCompetitionResponse` and the mirror.
    let dto: SolverCompetitionResponseDto =
        serde_json::from_value(fixture_payload(SOLVER_COMPETITION_FIXTURE))
            .expect("SolverCompetitionResponseDto must deserialize the native fixture");
    let rendered = serde_json::to_value(&dto).expect("SolverCompetitionResponseDto must serialize");
    assert_fixture_preserved(
        "SolverCompetitionResponseDto",
        SOLVER_COMPETITION_FIXTURE,
        &rendered,
    );
}

#[wasm_bindgen_test]
fn competition_order_status_dto_mirrors_native_shape() {
    // `CompetitionOrderStatus` renames `kind` to the wire `type` and nests
    // `SolverExecution` + `ExecutedAmounts`; it has no standalone golden fixture,
    // so pin the wire shape inline (under the same payload envelope the committed
    // fixtures use) so a rename, retype, or drop fails closed.
    const FIXTURE: &str = r#"{"payload":{"type":"solved","value":[{"solver":"0xabc","executedAmounts":{"sell":"1000","buy":"2000"}}]}}"#;
    let dto: CompetitionOrderStatusDto = serde_json::from_value(fixture_payload(FIXTURE))
        .expect("CompetitionOrderStatusDto must deserialize");
    let rendered = serde_json::to_value(&dto).expect("CompetitionOrderStatusDto must serialize");
    assert_fixture_preserved("CompetitionOrderStatusDto", FIXTURE, &rendered);
}
