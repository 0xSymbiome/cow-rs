//! Property coverage for orderbook request/response shape invariants.
//!
//! These properties pin cow-rs-specific serialization *logic* — quote-side
//! coercion, `validFor`/`validTo` exclusivity, EIP-1271-gated
//! `verificationGasLimit`, app-data document/hash composition, the trades
//! owner/order-uid filter, and fee normalization — across generated inputs.
//! Coverage uses `proptest` (shrinking + a committed regression file) to match
//! the crate's existing property-test convention.

#![allow(
    clippy::redundant_clone,
    clippy::too_many_lines,
    reason = "wire-shape property tests keep explicit request construction close to their assertions"
)]

mod common;

use cow_sdk_core::Amount;
use cow_sdk_orderbook::{
    BuyTokenDestination, OrderQuoteRequest, OrderQuoteResponse, OrderQuoteSide, OrdersQuery,
    PriceQuality, QuoteSigningScheme, QuoteValidity, SellTokenSource, SigningScheme, TradesQuery,
    calculate_total_fee,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

use crate::common::{
    sample_app_data_hash, sample_buy_token, sample_order_json, sample_order_uid, sample_owner,
    sample_quote_response_json,
};

const REGRESSION_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/invariant_contract.txt"
);

fn amount_strategy() -> impl Strategy<Value = Amount> {
    (1u64..1_000_000_000_000u64)
        .prop_map(|value| Amount::new(value.to_string()).expect("generated amount must parse"))
}

fn decimal_strategy() -> impl Strategy<Value = String> {
    (1u64..1_000_000_000_000u64).prop_map(|value| value.to_string())
}

fn price_quality_strategy() -> impl Strategy<Value = PriceQuality> {
    prop_oneof![
        Just(PriceQuality::Fast),
        Just(PriceQuality::Optimal),
        Just(PriceQuality::Verified),
    ]
}

fn signing_scheme_strategy() -> impl Strategy<Value = SigningScheme> {
    prop_oneof![
        Just(SigningScheme::Eip712),
        Just(SigningScheme::EthSign),
        Just(SigningScheme::Eip1271),
        Just(SigningScheme::PreSign),
    ]
}

fn sell_balance_strategy() -> impl Strategy<Value = SellTokenSource> {
    prop_oneof![
        Just(SellTokenSource::Erc20),
        Just(SellTokenSource::External),
        Just(SellTokenSource::Internal),
    ]
}

fn buy_balance_strategy() -> impl Strategy<Value = BuyTokenDestination> {
    prop_oneof![
        Just(BuyTokenDestination::Erc20),
        Just(BuyTokenDestination::Internal),
    ]
}

fn inline_app_data_strategy() -> impl Strategy<Value = String> {
    any::<u32>().prop_map(|nonce| {
        format!(
            "{{\"appCode\":\"cow-rs/orderbook-property\",\"metadata\":{{\"nonce\":{}}}}}",
            nonce % 10_000
        )
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSION_FILE))),
        ..ProptestConfig::default()
    })]

    #[test]
    fn quote_request_shape_roundtrips_without_side_coercion(
        is_sell in any::<bool>(),
        amount in amount_strategy(),
        price_quality in price_quality_strategy(),
        signing_scheme in signing_scheme_strategy(),
        sell_balance in sell_balance_strategy(),
        buy_balance in buy_balance_strategy(),
        set_receiver in any::<bool>(),
        valid_for in proptest::option::of(1u32..86_400),
        valid_to in proptest::option::of(any::<u32>()),
        timeout in proptest::option::of(1u64..60_000u64),
        verification_gas_limit in proptest::option::of(21_000u64..1_000_000u64),
        set_partially_fillable in any::<bool>(),
        set_onchain in any::<bool>(),
    ) {
        let side = if is_sell {
            OrderQuoteSide::sell(amount)
        } else {
            OrderQuoteSide::buy(amount)
        };
        let mut request =
            OrderQuoteRequest::new(sample_owner(), sample_buy_token(), sample_owner(), side)
                .with_price_quality(price_quality)
                .with_signing_scheme(signing_scheme)
                .with_sell_token_balance(sell_balance)
                .with_buy_token_balance(buy_balance);

        if set_receiver {
            request = request.with_receiver(sample_buy_token());
        }
        if let Some(seconds) = valid_for {
            request = request.with_valid_for(seconds);
        }
        if let Some(timestamp) = valid_to {
            request = request.with_valid_to(timestamp);
        }
        if let Some(milliseconds) = timeout {
            request = request.with_timeout(milliseconds);
        }
        if let Some(gas) = verification_gas_limit {
            request = request.with_verification_gas_limit(gas);
        }
        if set_partially_fillable {
            request = request.with_partially_fillable();
        }
        if set_onchain {
            request = request.with_onchain_order();
        }

        let value = serde_json::to_value(&request).expect("request serialization must succeed");

        if is_sell {
            prop_assert!(value.get("sellAmountBeforeFee").is_some());
            prop_assert!(value.get("buyAmountAfterFee").is_none());
            prop_assert_eq!(value.get("kind").and_then(serde_json::Value::as_str), Some("sell"));
        } else {
            prop_assert!(value.get("buyAmountAfterFee").is_some());
            prop_assert!(value.get("sellAmountBeforeFee").is_none());
            prop_assert_eq!(value.get("kind").and_then(serde_json::Value::as_str), Some("buy"));
        }

        prop_assert_eq!(
            value.get("receiver").is_some(),
            request.receiver.is_some(),
            "receiver presence must stay explicit through serialization"
        );
        prop_assert_eq!(
            value.get("validFor").is_some(),
            matches!(request.validity, QuoteValidity::ValidFor(_)),
            "validFor presence must match the typed validity variant"
        );
        prop_assert_eq!(
            value.get("validTo").is_some(),
            matches!(request.validity, QuoteValidity::ValidTo(_)),
            "validTo presence must match the typed validity variant"
        );
        prop_assert!(
            value.get("validFor").is_some() ^ value.get("validTo").is_some(),
            "exactly one of validFor or validTo must serialize"
        );
        prop_assert_eq!(
            value.get("timeout").is_some(),
            request.timeout.is_some(),
            "timeout presence must stay explicit through serialization"
        );
        prop_assert_eq!(
            value.get("verificationGasLimit").is_some(),
            matches!(request.signing_scheme, QuoteSigningScheme::Eip1271 { .. }),
            "verificationGasLimit serializes only for the EIP-1271 scheme"
        );

        let roundtrip: OrderQuoteRequest =
            serde_json::from_value(value).expect("request roundtrip must remain stable");
        prop_assert_eq!(roundtrip, request);
    }

    #[test]
    fn quote_request_app_data_and_pagination_shape_roundtrip_without_normalization(
        is_sell in any::<bool>(),
        amount in amount_strategy(),
        inline_app_data in proptest::option::of(inline_app_data_strategy()),
        set_app_data_hash in any::<bool>(),
        set_pagination in any::<bool>(),
        offset in any::<u32>(),
        limit in 1u32..5_000,
    ) {
        let side = if is_sell {
            OrderQuoteSide::sell(amount)
        } else {
            OrderQuoteSide::buy(amount)
        };
        let mut request =
            OrderQuoteRequest::new(sample_owner(), sample_buy_token(), sample_owner(), side);

        let app_data_hash = set_app_data_hash.then(sample_app_data_hash);
        if let Some(app_data) = inline_app_data.clone() {
            request = request.with_app_data(app_data);
        }
        if let Some(hash) = app_data_hash {
            request = request.with_app_data_hash(hash);
        }

        let value = serde_json::to_value(&request).expect("request serialization must succeed");
        let roundtrip: OrderQuoteRequest =
            serde_json::from_value(value.clone()).expect("request roundtrip must remain stable");
        prop_assert_eq!(&roundtrip, &request);

        // App-data wire shape after composition:
        // - a full document (when set) travels under `appData`;
        // - otherwise an explicit hash (when set) travels under `appData`;
        // - with neither set, `appData` is omitted and the orderbook applies
        //   its zero app-data hash default.
        let expected_app_data = match (inline_app_data.as_deref(), app_data_hash) {
            (Some(doc), _) => Some(doc.to_owned()),
            (None, Some(hash)) => Some(hash.to_hex_string()),
            (None, None) => None,
        };
        prop_assert_eq!(
            value.get("appData").and_then(serde_json::Value::as_str),
            expected_app_data.as_deref(),
            "appData carries the document if set, else the explicit hash, else is omitted"
        );
        prop_assert_eq!(
            value.get("appDataHash").is_some(),
            inline_app_data.is_some() && app_data_hash.is_some(),
            "appDataHash appears only alongside a full document"
        );

        let owner_request = if set_pagination {
            OrdersQuery::new(sample_owner())
                .with_offset(offset)
                .with_limit(limit)
        } else {
            OrdersQuery::new(sample_owner())
        };
        let owner_value =
            serde_json::to_value(&owner_request).expect("orders request must serialize");
        let owner_roundtrip: OrdersQuery =
            serde_json::from_value(owner_value).expect("orders request must deserialize");

        prop_assert_eq!(&owner_roundtrip, &owner_request);
        if owner_request.offset == 0 && owner_request.limit == 1_000 {
            prop_assert_eq!(&owner_request, &OrdersQuery::new(sample_owner()));
        }
    }

    #[test]
    fn trades_request_filter_xor_is_never_silently_normalized(
        set_owner in any::<bool>(),
        set_order_uid in any::<bool>(),
        offset in 0u32..500,
        limit in 1u32..100,
    ) {
        let owner = set_owner.then(sample_owner);
        let order_uid = set_order_uid.then(sample_order_uid);
        let request = TradesQuery::new(owner, order_uid)
            .with_offset(offset)
            .with_limit(limit);

        let value = serde_json::to_value(&request).expect("trades request must serialize");
        let roundtrip: TradesQuery =
            serde_json::from_value(value).expect("trades request must deserialize");

        prop_assert_eq!(request.is_valid(), owner.is_some() ^ order_uid.is_some());
        prop_assert_eq!(&roundtrip, &request);
        prop_assert_eq!(roundtrip.is_valid(), request.is_valid());
    }

    #[test]
    fn fee_normalization_trims_leading_zeroes_across_generated_decimal_inputs(
        value in decimal_strategy(),
        leading_zeroes in 0usize..3,
    ) {
        let padded = format!("{}{}", "0".repeat(leading_zeroes), value);
        let expected = value
            .parse::<u128>()
            .expect("generated decimal must parse")
            .to_string();
        let total_fee =
            calculate_total_fee(Some(&padded)).expect("generated decimal normalization must remain valid");

        prop_assert_eq!(
            total_fee,
            Amount::new(expected).expect("expected amount must parse")
        );
    }
}

#[test]
fn malformed_payloads_fail_closed_in_decoding_and_transforms() {
    // A malformed app-data hash in a quote response fails closed.
    let mut response = sample_quote_response_json();
    response["quote"]["appData"] = serde_json::json!("not-a-hash");
    response["quote"]
        .as_object_mut()
        .expect("quote fixture must remain an object")
        .remove("appDataHash");
    let error = serde_json::from_value::<OrderQuoteResponse>(response)
        .expect_err("malformed app-data hashes must fail closed");
    assert!(!error.to_string().is_empty());

    // An unsupported order kind fails closed.
    let mut response = sample_quote_response_json();
    response["quote"]["kind"] = serde_json::json!("hold");
    let error = serde_json::from_value::<OrderQuoteResponse>(response)
        .expect_err("unsupported order kinds must fail closed");
    assert!(!error.to_string().is_empty());

    // Invalid `executedFee` strings fail at the typed wire boundary.
    for invalid in ["abc", "12z"] {
        let mut order_json = sample_order_json(&sample_order_uid());
        order_json["executedFee"] = serde_json::json!(invalid);
        let error = serde_json::from_value::<cow_sdk_orderbook::Order>(order_json)
            .expect_err("invalid executedFee must fail at typed wire boundary");
        assert!(error.to_string().contains("amount"));
    }
}
