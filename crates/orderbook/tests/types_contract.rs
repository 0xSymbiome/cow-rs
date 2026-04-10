mod common;

use cow_sdk_orderbook::{
    ApiContextOverride, CowEnv, GetOrdersRequest, GetTradesRequest, OrderBalance, OrderCreation,
    OrderKind, OrderQuoteRequest, PriceQuality, QuoteSide, SigningScheme, SupportedChainId,
};
use serde_json::json;

use crate::common::{
    default_context, sample_app_data_hash, sample_buy_token, sample_order_uid, sample_owner,
    sample_quote_response_json, sample_signature,
};

#[test]
fn quote_request_defaults_match_transport_contract() {
    let request = OrderQuoteRequest::new(
        sample_owner(),
        sample_buy_token(),
        sample_owner(),
        QuoteSide::sell("1000000"),
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
    let override_context = ApiContextOverride {
        chain_id: Some(SupportedChainId::Mainnet),
        env: Some(CowEnv::Staging),
        base_urls: None,
        api_key: Some("partner-key".to_owned()),
    };

    let request = OrderQuoteRequest::new(
        sample_owner(),
        sample_buy_token(),
        sample_owner(),
        QuoteSide::buy("900000"),
    )
    .with_app_data_hash(sample_app_data_hash())
    .with_valid_for(1_800)
    .with_price_quality(PriceQuality::Optimal)
    .with_signing_scheme(SigningScheme::Eip1271)
    .with_sell_token_balance(OrderBalance::External)
    .with_buy_token_balance(OrderBalance::Internal)
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
    assert_eq!(override_context.api_key.as_deref(), Some("partner-key"));
}

#[test]
fn orders_and_trades_requests_keep_upstream_defaults() {
    let owner = sample_owner();
    let orders = GetOrdersRequest::new(owner.clone());
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
    let invalid_both = GetTradesRequest {
        owner: Some(sample_owner()),
        order_uid: Some(sample_order_uid()),
        offset: 0,
        limit: 10,
    };
    let invalid_neither = GetTradesRequest {
        owner: None,
        order_uid: None,
        offset: 0,
        limit: 10,
    };

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
    assert_eq!(order_value["from"], json!(sample_owner().as_str()));
}

#[test]
fn quote_response_accepts_full_app_data_echo_when_hash_is_present() {
    let response = serde_json::from_value::<cow_sdk_orderbook::OrderQuoteResponse>(json!({
        "quote": {
            "sellToken": sample_owner().as_str(),
            "buyToken": sample_buy_token().as_str(),
            "receiver": sample_owner().as_str(),
            "sellAmount": "1000",
            "buyAmount": "900",
            "validTo": 1700000000,
            "appData": "{\"appCode\":\"cow-rs/wasm-console\",\"version\":\"1.14.0\"}",
            "appDataHash": sample_app_data_hash().as_str(),
            "feeAmount": "10",
            "kind": "sell",
            "partiallyFillable": false,
            "sellTokenBalance": "erc20",
            "buyTokenBalance": "erc20"
        },
        "from": sample_owner().as_str(),
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
