mod common;

use cow_sdk_orderbook::{
    Amount, ApiContextOverride, BuyTokenDestination, CowEnv, GetOrdersRequest, GetTradesRequest,
    OrderCreation, OrderKind, OrderQuoteRequest, OrderbookError, PriceQuality, QuoteSide,
    SellTokenSource, SigningScheme, SupportedChainId,
};
use serde_json::json;

use crate::common::{
    default_context, sample_app_data_hash, sample_buy_token, sample_order_uid, sample_owner,
    sample_quote_response_json, sample_signature,
};

fn amount(value: &str) -> Amount {
    Amount::new(value).expect("test amount literal must be valid")
}

#[test]
fn quote_request_defaults_match_transport_contract() {
    let request = OrderQuoteRequest::new(
        sample_owner(),
        sample_buy_token(),
        sample_owner(),
        QuoteSide::sell(amount("1000000")),
    );

    let value = serde_json::to_value(&request).expect("quote request must serialize");

    assert_eq!(value["kind"], json!("sell"));
    assert_eq!(value["sellAmountBeforeFee"], json!("1000000"));
    assert_eq!(
        value["appData"],
        json!("0x0000000000000000000000000000000000000000000000000000000000000000")
    );
    assert!(value.get("validFor").is_none());
    assert_eq!(value["priceQuality"], json!("verified"));
    assert_eq!(value["signingScheme"], json!("eip712"));
    assert_eq!(value["sellTokenBalance"], json!("erc20"));
    assert_eq!(value["buyTokenBalance"], json!("erc20"));
}

#[test]
fn quote_request_supports_buy_side_and_context_overrides() {
    let override_context = ApiContextOverride::new()
        .with_chain_id(SupportedChainId::Mainnet)
        .with_env(CowEnv::Staging)
        .with_base_urls(std::collections::BTreeMap::from([(
            u64::from(SupportedChainId::Mainnet),
            "https://user:pass@example.test/path?apiKey=secret-token".to_owned(),
        )]))
        .with_api_key("partner-key".to_owned().into());

    let request = OrderQuoteRequest::new(
        sample_owner(),
        sample_buy_token(),
        sample_owner(),
        QuoteSide::buy(amount("900000")),
    )
    .with_app_data_hash(sample_app_data_hash())
    .with_valid_for(1_800)
    .with_price_quality(PriceQuality::Optimal)
    .with_signing_scheme(SigningScheme::Eip1271)
    .with_sell_token_balance(SellTokenSource::External)
    .with_buy_token_balance(BuyTokenDestination::Internal)
    .with_verification_gas_limit(0)
    .with_timeout(2_500)
    .with_onchain_order();

    assert!(request.is_buy());
    assert!(request.is_valid());
    let value = serde_json::to_value(&request).expect("quote request must serialize");
    assert_eq!(value["validFor"], json!(1_800));
    assert_eq!(value["verificationGasLimit"], json!(0));
    assert_eq!(override_context.env, Some(CowEnv::Staging));
    assert_eq!(override_context.chain_id, Some(SupportedChainId::Mainnet));
    assert_eq!(
        override_context
            .api_key
            .as_ref()
            .map(|value| value.as_inner().as_str()),
        Some("partner-key")
    );

    let debug = format!("{override_context:?}");
    assert!(debug.contains("ApiContextOverride"));
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("partner-key"));
    assert!(!debug.contains("user:pass"));
    assert!(!debug.contains("apiKey=secret-token"));
    assert!(!debug.contains("example.test"));

    let override_value =
        serde_json::to_value(&override_context).expect("context override must serialize");
    assert_eq!(override_value["apiKey"], json!("[redacted]"));
    assert_eq!(override_value["baseUrls"]["1"], json!("[redacted]"));
}

#[test]
fn quote_request_validate_rejects_incompatible_onchain_ecdsa_signing_pairs() {
    for signing_scheme in [SigningScheme::Eip712, SigningScheme::EthSign] {
        let error = OrderQuoteRequest::new(
            sample_owner(),
            sample_buy_token(),
            sample_owner(),
            QuoteSide::sell(amount("1000000")),
        )
        .with_signing_scheme(signing_scheme)
        .with_onchain_order()
        .validate()
        .expect_err("ECDSA on-chain quote request must reject locally");

        assert!(matches!(
            error,
            OrderbookError::IncompatibleSigningScheme {
                signing_scheme: rejected_scheme,
                onchain_order: true,
            } if rejected_scheme == signing_scheme
        ));
    }
}

#[test]
fn quote_request_validate_accepts_services_signing_scheme_pairs() {
    for (signing_scheme, onchain_order) in [
        (SigningScheme::Eip712, false),
        (SigningScheme::EthSign, false),
        (SigningScheme::Eip1271, false),
        (SigningScheme::Eip1271, true),
        (SigningScheme::PreSign, false),
        (SigningScheme::PreSign, true),
    ] {
        let mut request = OrderQuoteRequest::new(
            sample_owner(),
            sample_buy_token(),
            sample_owner(),
            QuoteSide::sell(amount("1000000")),
        )
        .with_signing_scheme(signing_scheme);
        if onchain_order {
            request = request.with_onchain_order();
        }

        request
            .validate()
            .expect("services-compatible signing pair must validate locally");
    }
}

#[test]
fn quote_request_validate_rejects_verification_gas_limit_without_eip1271() {
    let error = OrderQuoteRequest::new(
        sample_owner(),
        sample_buy_token(),
        sample_owner(),
        QuoteSide::sell(amount("1000000")),
    )
    .with_verification_gas_limit(27_000)
    .validate()
    .expect_err("verificationGasLimit must be reserved for eip1271");

    assert!(matches!(
        error,
        OrderbookError::InvalidQuoteRequest {
            field: "verificationGasLimit",
            ..
        }
    ));
}

#[test]
fn orders_and_trades_requests_keep_upstream_defaults() {
    let owner = sample_owner();
    let orders = GetOrdersRequest::new(owner);
    let trades_by_owner = GetTradesRequest::by_owner(owner);
    let trades_by_uid = GetTradesRequest::by_order_uid(sample_order_uid());

    assert_eq!(orders.offset, 0);
    assert_eq!(orders.limit, 1_000);
    assert!(trades_by_owner.is_valid());
    assert!(trades_by_uid.is_valid());
    assert_eq!(trades_by_owner.offset, 0);
    assert_eq!(trades_by_owner.limit, 10);
}

#[test]
fn trades_request_rejects_owner_and_uid_or_neither() {
    let invalid_both = GetTradesRequest::new(Some(sample_owner()), Some(sample_order_uid()));
    let invalid_neither = GetTradesRequest::new(None, None);

    assert!(!invalid_both.is_valid());
    assert!(!invalid_neither.is_valid());
}

#[test]
fn order_creation_from_quote_keeps_quote_shape_and_quote_id() {
    let quote_response = serde_json::from_value::<cow_sdk_orderbook::OrderQuoteResponse>(
        sample_quote_response_json(),
    )
    .expect("quote response fixture must deserialize");
    let order = OrderCreation::from_quote(
        &quote_response.quote,
        sample_owner(),
        None,
        SigningScheme::EthSign,
        sample_signature(),
    )
    .with_quote_id(quote_response.id.expect("fixture has quote id"));

    assert_eq!(order.kind, OrderKind::Sell);
    assert_eq!(order.quote_id, Some(42));
    assert_eq!(order.signing_scheme, SigningScheme::EthSign);
    assert!(order.app_data.is_none());
    assert_eq!(order.app_data_hash, Some(sample_app_data_hash()));

    let quote_value = serde_json::to_value(&quote_response.quote).expect("quote serializes");
    let order_value = serde_json::to_value(&order).expect("order creation serializes");
    assert!(quote_value.get("signature").is_none());
    assert!(quote_value.get("from").is_none());
    assert_eq!(order_value["signature"], json!(sample_signature()));
    assert_eq!(order_value["from"], json!(sample_owner().to_hex_string()));
}

#[test]
fn order_creation_serialize_routes_app_data_combinations_to_services_variants() {
    // The cow `OrderCreation::Serialize` impl routes the four
    // `(app_data, app_data_hash)` combinations onto the three services
    // `OrderCreationAppData` untagged-enum variants. This test pins
    // each wire shape against the services contract documented at
    // `cowprotocol/services` `crates/model/src/order.rs:439-461`:
    //
    // - `Both`  -> `appData` is the full document string, `appDataHash`
    //              is the explicit hash.
    // - `Hash`  -> `appData` carries the hash hex string (no separate
    //              `appDataHash` field).
    // - `Full`  -> `appData` is the full document string; services
    //              derives the hash.
    //
    // The `(None, None)` combination intentionally emits no app-data
    // field. Services rejects the resulting request because no variant
    // matches; that is the documented programmer-error surface
    // (callers must attach app-data via `with_app_data` or
    // `with_app_data_hash`).

    let base = || {
        OrderCreation::new(
            sample_owner(),
            sample_buy_token(),
            amount("1000000"),
            amount("900000"),
            1_700_000_000,
            OrderKind::Sell,
            SigningScheme::Eip712,
            sample_signature(),
            sample_owner(),
        )
    };

    let full_doc = "{\"version\":\"1.0.0\",\"metadata\":{}}";
    let hash = sample_app_data_hash();
    let hash_hex = hash.to_hex_string();

    // (None, None) — both fields omitted. Services rejects; the test
    // pins the absence so a future change cannot silently restore the
    // pre-fix `appDataHash`-only emission.
    let neither = serde_json::to_value(base()).expect("OrderCreation serializes");
    assert!(neither.get("appData").is_none());
    assert!(neither.get("appDataHash").is_none());

    // (Some(s), None) — services `Full` variant.
    let full_only =
        serde_json::to_value(base().with_app_data(full_doc)).expect("OrderCreation serializes");
    assert_eq!(full_only["appData"], json!(full_doc));
    assert!(full_only.get("appDataHash").is_none());

    // (None, Some(h)) — services `Hash` variant. The hash hex string
    // lives under the `appData` key, NOT `appDataHash`.
    let hash_only =
        serde_json::to_value(base().with_app_data_hash(hash)).expect("OrderCreation serializes");
    assert_eq!(hash_only["appData"], json!(hash_hex));
    assert!(hash_only.get("appDataHash").is_none());

    // (Some(s), Some(h)) — services `Both` variant.
    let both = serde_json::to_value(base().with_app_data(full_doc).with_app_data_hash(hash))
        .expect("OrderCreation serializes");
    assert_eq!(both["appData"], json!(full_doc));
    assert_eq!(both["appDataHash"], json!(hash_hex));
}

#[test]
fn order_creation_from_quote_serialize_emits_services_hash_variant() {
    // `from_quote` produces `(app_data: None, app_data_hash: Some(quote.app_data))`
    // because the quote response carries only the hash; the full
    // app-data document is not part of the quote shape. The wire
    // emission must match the services `Hash` variant: the hash hex
    // string under the `appData` key, no `appDataHash` field.

    let quote_response = serde_json::from_value::<cow_sdk_orderbook::OrderQuoteResponse>(
        sample_quote_response_json(),
    )
    .expect("quote response fixture must deserialize");
    let order = OrderCreation::from_quote(
        &quote_response.quote,
        sample_owner(),
        None,
        SigningScheme::EthSign,
        sample_signature(),
    );

    let wire = serde_json::to_value(&order).expect("OrderCreation serializes");
    let expected_hash = sample_app_data_hash().to_hex_string();
    assert_eq!(wire["appData"], json!(expected_hash));
    assert!(wire.get("appDataHash").is_none());
}

#[test]
fn order_creation_full_balance_check_is_opt_in_on_the_wire() {
    let order = OrderCreation::new(
        sample_owner(),
        sample_buy_token(),
        amount("1000000"),
        amount("900000"),
        1_700_000_000,
        OrderKind::Sell,
        SigningScheme::Eip712,
        sample_signature(),
        sample_owner(),
    );

    let default_value = serde_json::to_value(&order).expect("order creation serializes");
    assert!(default_value.get("fullBalanceCheck").is_none());

    let checked_value = serde_json::to_value(order.clone().with_full_balance_check(true))
        .expect("checked order creation serializes");
    assert_eq!(checked_value["fullBalanceCheck"], json!(true));

    let false_value = serde_json::to_value(order.with_full_balance_check(false))
        .expect("unchecked order creation serializes");
    assert!(false_value.get("fullBalanceCheck").is_none());
}

#[test]
fn quote_response_accepts_full_app_data_echo_when_hash_is_present() {
    let response = serde_json::from_value::<cow_sdk_orderbook::OrderQuoteResponse>(json!({
        "quote": {
            "sellToken": sample_owner().to_hex_string(),
            "buyToken": sample_buy_token().to_hex_string(),
            "receiver": sample_owner().to_hex_string(),
            "sellAmount": "1000",
            "buyAmount": "900",
            "validTo": 1_700_000_000,
            "appData": "{\"appCode\":\"cow-rs/wasm-console\",\"version\":\"1.14.0\"}",
            "appDataHash": sample_app_data_hash().to_hex_string(),
            "feeAmount": "10",
            "kind": "sell",
            "partiallyFillable": false,
            "sellTokenBalance": "erc20",
            "buyTokenBalance": "erc20"
        },
        "from": sample_owner().to_hex_string(),
        "expiration": "2026-04-08T10:00:00Z",
        "id": 42,
        "verified": true
    }))
    .expect("quote response with full app-data echo must deserialize");

    assert_eq!(response.quote.app_data, sample_app_data_hash());
}

#[test]
fn core_api_context_resolution_remains_available_to_orderbook() {
    let context = default_context(SupportedChainId::GnosisChain, CowEnv::Prod);

    assert_eq!(
        context
            .resolved_base_url()
            .expect("gnosis prod base url should resolve"),
        "https://api.cow.fi/xdai"
    );
}
