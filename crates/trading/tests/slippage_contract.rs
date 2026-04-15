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
    SwapAdvancedSettings, partner_fee_bps, resolve_slippage_suggestion, sanitize_protocol_fee_bps,
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
        Ok(SlippageToleranceResponse {
            slippage_bps: self.response,
        })
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
    assert!(error.contains("Fee amount must be non-negative: -100"));
}

#[test]
fn slippage_bps_clamps_to_expected_bounds() {
    let trader = cow_sdk_trading::QuoterParameters {
        chain_id: SupportedChainId::Sepolia,
        app_code: "0x007".to_owned(),
        account: address(OWNER),
        env: None,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
    };
    let trade = sample_trade_parameters(OrderKind::Sell);

    let zero_quote = cow_sdk_orderbook::OrderQuoteResponse {
        quote: QuoteData {
            sell_token: address(WETH),
            buy_token: address(COW),
            receiver: Some(address(OWNER)),
            sell_amount: "1".to_owned(),
            buy_amount: "1".to_owned(),
            valid_to: 1,
            app_data: crate::common::app_data_hash(),
            fee_amount: "0".to_owned(),
            kind: OrderKind::Sell,
            partially_fillable: false,
            sell_token_balance: cow_sdk_core::OrderBalance::Erc20,
            buy_token_balance: cow_sdk_core::OrderBalance::Erc20,
        },
        from: Some(address(OWNER)),
        expiration: "2025-01-21T12:55:14.799709609Z".to_owned(),
        id: Some(1),
        verified: true,
        protocol_fee_bps: None,
    };
    let huge_fee_quote = cow_sdk_orderbook::OrderQuoteResponse {
        quote: QuoteData {
            fee_amount: "1000000000000000000000".to_owned(),
            ..zero_quote.quote.clone()
        },
        ..zero_quote.clone()
    };

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
        partner_fee_bps(Some(&PartnerFee::from(PartnerFeePolicy::volume(
            42,
            address(crate::common::ALT_RECEIVER),
        )))),
        Some(42)
    );
    assert_eq!(
        partner_fee_bps(Some(&PartnerFee::from(vec![
            PartnerFeePolicy::price_improvement(12, 100, address(crate::common::ALT_RECEIVER)),
            PartnerFeePolicy::volume(55, address(crate::common::ALT_RECEIVER)),
        ]))),
        Some(55)
    );
    assert_eq!(
        partner_fee_bps(Some(&PartnerFee::from(PartnerFeePolicy::surplus(
            250,
            100,
            address(crate::common::ALT_RECEIVER),
        )))),
        None
    );
    assert_eq!(partner_fee_bps(None), None);
}

#[tokio::test]
async fn resolve_slippage_suggestion_skips_provider_for_fast_quotes_and_uses_provider_for_optimal()
{
    let trade = sample_trade_parameters(OrderKind::Sell);
    let trader = cow_sdk_trading::QuoterParameters {
        chain_id: SupportedChainId::Sepolia,
        app_code: "0x007".to_owned(),
        account: address(OWNER),
        env: None,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
    };
    let quote = sell_quote_response();
    let fast_calls = Arc::new(AtomicUsize::new(0));
    let fast_settings = SwapAdvancedSettings {
        quote_request: Some(QuoteRequestOverride {
            price_quality: Some(PriceQuality::Fast),
            ..QuoteRequestOverride::default()
        }),
        slippage_suggester: Some(Arc::new(CountingProvider {
            calls: fast_calls.clone(),
            response: Some(200),
        })),
        ..SwapAdvancedSettings::default()
    };

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
    let optimal_settings = SwapAdvancedSettings {
        slippage_suggester: Some(Arc::new(CountingProvider {
            calls: optimal_calls.clone(),
            response: Some(200),
        })),
        ..SwapAdvancedSettings::default()
    };
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
