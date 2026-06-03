mod common;

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;

use cow_sdk_core::{Amount, OrderKind, SupportedChainId};
use cow_sdk_orderbook::{PriceQuality, QuoteData};
use cow_sdk_trading::{
    MAX_SLIPPAGE_BPS, PartnerFee, PartnerFeePolicy, QuoteRequestOverride,
    SlippageSuggestionProvider, SlippageToleranceRequest, SlippageToleranceResponse,
    TradeAdvancedSettings, partner_fee_bps, resolve_slippage_suggestion, sanitize_protocol_fee_bps,
    suggest_slippage_bps, suggest_slippage_from_fee, suggest_slippage_from_volume,
};

use crate::common::{COW, OWNER, WETH, address, sample_trade_parameters, sell_quote_response};

struct CountingProvider {
    calls: Arc<AtomicUsize>,
    response: Option<u32>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl SlippageSuggestionProvider for CountingProvider {
    async fn get_slippage_suggestion(
        &self,
        _request: SlippageToleranceRequest,
    ) -> Result<SlippageToleranceResponse, cow_sdk_trading::TradingError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut response = SlippageToleranceResponse::new();
        if let Some(bps) = self.response {
            response = response.with_slippage_bps(bps);
        }
        Ok(response)
    }
}

#[test]
fn slippage_helpers_follow_upstream_fee_and_volume_examples() {
    assert_eq!(
        suggest_slippage_from_fee("20", 50.0).expect("fee suggestion should work"),
        Amount::new("10").expect("test amount literal must be valid")
    );
    assert_eq!(
        suggest_slippage_from_volume(true, "20", "15", 50.0)
            .expect("sell-volume suggestion should work"),
        Amount::new("8").expect("test amount literal must be valid")
    );
    assert_eq!(
        suggest_slippage_from_volume(false, "20", "15", 25.0)
            .expect("buy-volume suggestion should work"),
        Amount::new("5").expect("test amount literal must be valid")
    );

    let error = suggest_slippage_from_fee("-100", 50.0)
        .expect_err("negative fee must fail")
        .to_string();
    assert!(
        error.contains("feeAmount") && error.contains("non-negative"),
        "negative fee error must name the field and reason, got: {error}"
    );
}

#[test]
fn slippage_bps_clamps_to_expected_bounds() {
    let trader =
        cow_sdk_trading::QuoterParameters::new(SupportedChainId::Sepolia, "0x007", address(OWNER))
            .expect("app code should validate");
    let trade = sample_trade_parameters(OrderKind::Sell);

    let zero_quote_data = QuoteData::new(
        address(WETH),
        address(COW),
        Amount::new("1").expect("test amount literal must be valid"),
        Amount::new("1").expect("test amount literal must be valid"),
        1,
        crate::common::app_data_hash(),
        OrderKind::Sell,
    )
    .with_network_cost_amount(Amount::ZERO)
    .with_receiver(address(OWNER));
    let zero_quote = cow_sdk_orderbook::OrderQuoteResponse::new(
        zero_quote_data,
        "2025-01-21T12:55:14.799709609Z",
        true,
    )
    .with_from(address(OWNER))
    .with_id(1);
    let huge_fee_quote_data = QuoteData::new(
        address(WETH),
        address(COW),
        Amount::new("1").expect("test amount literal must be valid"),
        Amount::new("1").expect("test amount literal must be valid"),
        1,
        crate::common::app_data_hash(),
        OrderKind::Sell,
    )
    .with_network_cost_amount(
        Amount::new("1000000000000000000000").expect("test amount literal must be valid"),
    )
    .with_receiver(address(OWNER));
    let huge_fee_quote = cow_sdk_orderbook::OrderQuoteResponse::new(
        huge_fee_quote_data,
        "2025-01-21T12:55:14.799709609Z",
        true,
    )
    .with_from(address(OWNER))
    .with_id(1);

    assert_eq!(
        suggest_slippage_bps(&zero_quote, &trade, &trader, false, None)
            .expect("non-ethflow zero slippage should work"),
        0
    );
    assert_eq!(
        suggest_slippage_bps(&zero_quote, &trade, &trader, true, None)
            .expect("ethflow lower clamp should work"),
        50
    );
    assert_eq!(
        suggest_slippage_bps(&huge_fee_quote, &trade, &trader, false, None)
            .expect("max clamp should work"),
        MAX_SLIPPAGE_BPS
    );
}

#[test]
fn slippage_clamps_at_eth_flow_default_for_eth_flow_orders() {
    let trader =
        cow_sdk_trading::QuoterParameters::new(SupportedChainId::Sepolia, "0x007", address(OWNER))
            .expect("app code should validate");
    let trade = sample_trade_parameters(OrderKind::Sell);
    let quote_data = QuoteData::new(
        address(WETH),
        address(COW),
        Amount::new("1").expect("test amount literal must be valid"),
        Amount::new("1").expect("test amount literal must be valid"),
        1,
        crate::common::app_data_hash(),
        OrderKind::Sell,
    )
    .with_network_cost_amount(Amount::ZERO)
    .with_receiver(address(OWNER));
    let quote = cow_sdk_orderbook::OrderQuoteResponse::new(
        quote_data,
        "2025-01-21T12:55:14.799709609Z",
        true,
    )
    .with_from(address(OWNER))
    .with_id(1);

    assert_eq!(
        suggest_slippage_bps(&quote, &trade, &trader, true, None)
            .expect("EthFlow zero-cost slippage suggestion must clamp"),
        cow_sdk_trading::DEFAULT_SLIPPAGE_BPS,
    );
}

#[test]
fn protocol_fee_sanitization_accepts_only_finite_supported_values() {
    assert_eq!(sanitize_protocol_fee_bps(None), None);
    assert_eq!(sanitize_protocol_fee_bps(Some("not-a-number")), None);
    assert_eq!(sanitize_protocol_fee_bps(Some("0")), None);
    assert_eq!(sanitize_protocol_fee_bps(Some("0.00001")), None);
    assert_eq!(sanitize_protocol_fee_bps(Some("0.0001")), Some(0.0001));
    assert_eq!(sanitize_protocol_fee_bps(Some("-1")), None);
    assert_eq!(sanitize_protocol_fee_bps(Some("inf")), None);
    assert_eq!(sanitize_protocol_fee_bps(Some("1.25")), Some(1.25));
}

#[test]
fn partner_fee_extraction_prefers_supported_object_and_array_shapes() {
    assert_eq!(
        partner_fee_bps(Some(&PartnerFee::from(
            PartnerFeePolicy::volume(42, address(crate::common::ALT_RECEIVER))
                .expect("volume policy must validate"),
        ))),
        Some(42)
    );
    assert_eq!(
        partner_fee_bps(Some(&PartnerFee::from(vec![
            PartnerFeePolicy::price_improvement(12, 100, address(crate::common::ALT_RECEIVER))
                .expect("price-improvement policy must validate"),
            PartnerFeePolicy::volume(55, address(crate::common::ALT_RECEIVER))
                .expect("volume policy must validate"),
        ]))),
        Some(55)
    );
    assert_eq!(
        partner_fee_bps(Some(&PartnerFee::from(
            PartnerFeePolicy::surplus(250, 100, address(crate::common::ALT_RECEIVER))
                .expect("surplus policy must validate"),
        ))),
        None
    );
    assert_eq!(partner_fee_bps(None), None);
}

#[tokio::test]
async fn resolve_slippage_suggestion_skips_provider_for_fast_quotes_and_uses_provider_for_optimal()
{
    let trade = sample_trade_parameters(OrderKind::Sell);
    let trader =
        cow_sdk_trading::QuoterParameters::new(SupportedChainId::Sepolia, "0x007", address(OWNER))
            .expect("app code should validate");
    let quote = sell_quote_response();
    let fast_calls = Arc::new(AtomicUsize::new(0));
    let fast_settings = TradeAdvancedSettings::new()
        .with_quote_request(QuoteRequestOverride::new().with_price_quality(PriceQuality::Fast))
        .with_slippage_suggester(Arc::new(CountingProvider {
            calls: fast_calls.clone(),
            response: Some(200),
        }));

    let fast = resolve_slippage_suggestion(
        SupportedChainId::Sepolia,
        &trade,
        &trader,
        &quote,
        false,
        Some(&fast_settings),
    )
    .await
    .expect("fast slippage resolution should succeed");
    assert_eq!(fast_calls.load(Ordering::SeqCst), 0);
    assert!(fast.slippage_bps.is_some());

    let optimal_calls = Arc::new(AtomicUsize::new(0));
    let optimal_settings =
        TradeAdvancedSettings::new().with_slippage_suggester(Arc::new(CountingProvider {
            calls: optimal_calls.clone(),
            response: Some(200),
        }));
    let optimal = resolve_slippage_suggestion(
        SupportedChainId::Sepolia,
        &trade,
        &trader,
        &quote,
        false,
        Some(&optimal_settings),
    )
    .await
    .expect("optimal slippage resolution should succeed");

    assert_eq!(optimal_calls.load(Ordering::SeqCst), 1);
    assert_eq!(optimal.slippage_bps, Some(269));
}
