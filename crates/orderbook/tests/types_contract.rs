mod common;

use cow_sdk_core::OrderData;
use cow_sdk_orderbook::{
    Address, Amount, ApiContextOverride, AppDataHash, BuyTokenDestination, CowEnv, OrderCreation,
    OrderKind, OrderQuoteRequest, OrderQuoteSide, OrdersQuery, PriceQuality, QuoteAppData,
    QuoteSigningScheme, SellTokenSource, SigningScheme, SupportedChainId, TradesQuery,
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
fn order_creation_from_signed_mirrors_the_signed_order() {
    let sell_token = Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let buy_token = Address::new("0x2222222222222222222222222222222222222222").unwrap();
    let receiver = Address::new("0x3333333333333333333333333333333333333333").unwrap();
    let from = Address::new("0x4444444444444444444444444444444444444444").unwrap();
    let app_data_hash =
        AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap();

    let order_to_sign = OrderData::new(
        sell_token,
        buy_token,
        receiver,
        amount("1000000000000000000"),
        amount("900000000"),
        1_700_000_000,
        app_data_hash,
        amount("0"),
        OrderKind::Sell,
        true,
        SellTokenSource::External,
        BuyTokenDestination::Internal,
    );

    let creation = OrderCreation::from_signed(
        &order_to_sign,
        SigningScheme::Eip712,
        "0xsignature",
        from,
        Some("{\"version\":\"1.0.0\"}".to_owned()),
        Some(99),
    );

    // Every signed economic field is copied verbatim from the signing order.
    assert_eq!(creation.sell_token, sell_token);
    assert_eq!(creation.buy_token, buy_token);
    assert_eq!(creation.receiver, Some(receiver));
    assert_eq!(creation.sell_amount, amount("1000000000000000000"));
    assert_eq!(creation.buy_amount, amount("900000000"));
    assert_eq!(creation.valid_to, 1_700_000_000);
    assert_eq!(
        creation.app_data.as_deref(),
        Some("{\"version\":\"1.0.0\"}")
    );
    // The wire hash is taken from the signed order's `app_data` commitment, not
    // a separate caller-supplied value that could diverge from what was signed.
    assert_eq!(creation.app_data_hash, Some(app_data_hash));
    assert_eq!(creation.kind, OrderKind::Sell);
    assert!(creation.partially_fillable);
    assert_eq!(creation.sell_token_balance, SellTokenSource::External);
    assert_eq!(creation.buy_token_balance, BuyTokenDestination::Internal);
    assert_eq!(creation.signing_scheme, SigningScheme::Eip712);
    assert_eq!(creation.signature, "0xsignature");
    assert_eq!(creation.from, from);
    assert_eq!(creation.quote_id, Some(99));

    // The order-level fee is always wired as "0" on submission, independent of
    // any fee carried on the signing order.
    let value = serde_json::to_value(&creation).expect("submission payload must serialize");
    assert_eq!(value["feeAmount"], json!("0"));
}

#[test]
fn quote_request_defaults_match_transport_contract() {
    let request = OrderQuoteRequest::new(
        sample_owner(),
        sample_buy_token(),
        sample_owner(),
        OrderQuoteSide::sell(amount("1000000")),
    );

    let value = serde_json::to_value(&request).expect("quote request must serialize");

    assert_eq!(value["kind"], json!("sell"));
    assert_eq!(value["sellAmountBeforeFee"], json!("1000000"));
    // A default request attaches no app-data; the orderbook treats an omitted
    // app-data field as the zero app-data hash, so neither key is emitted.
    assert!(value.get("appData").is_none());
    assert!(value.get("appDataHash").is_none());
    // A default request now carries the protocol 30-minute relative validity
    // explicitly on the wire, matching the orderbook quote contract.
    assert_eq!(value["validFor"], json!(1_800));
    assert!(value.get("validTo").is_none());
    assert_eq!(value["priceQuality"], json!("optimal"));
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
        OrderQuoteSide::buy(amount("900000")),
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
fn quote_signing_scheme_rejects_incompatible_onchain_ecdsa_wire_pairs() {
    // ECDSA on-chain is now unrepresentable in the typed builder; the wire
    // try_from rejects the invalid combination on deserialization instead.
    for scheme in ["eip712", "ethsign"] {
        let json = format!(r#"{{"signingScheme":"{scheme}","onchainOrder":true}}"#);
        let error = serde_json::from_str::<QuoteSigningScheme>(&json)
            .expect_err("ECDSA on-chain signing scheme must reject on the wire");
        assert!(
            error.to_string().contains("on-chain"),
            "error should explain the ECDSA on-chain rejection: {error}"
        );
    }
}

#[test]
fn quote_signing_scheme_rejects_verification_gas_limit_without_eip1271() {
    // A verification gas limit only belongs to EIP-1271; on any other scheme
    // the wire try_from rejects it on deserialization.
    let error = serde_json::from_str::<QuoteSigningScheme>(
        r#"{"signingScheme":"eip712","verificationGasLimit":27000}"#,
    )
    .expect_err("verificationGasLimit must be reserved for eip1271");

    assert!(
        error.to_string().contains("verificationGasLimit"),
        "error should explain the verificationGasLimit rejection: {error}"
    );
}

#[test]
fn quote_request_app_data_routes_to_server_valid_wire_shapes() {
    let hash = sample_app_data_hash();
    let hash_wire = serde_json::to_value(hash).expect("hash serializes");
    let base = || {
        OrderQuoteRequest::new(
            sample_owner(),
            sample_buy_token(),
            sample_owner(),
            OrderQuoteSide::sell(amount("1000000")),
        )
    };

    // Full only -> {"appData": <document>}, no appDataHash.
    let full = base().with_app_data("{\"version\":\"1.4.0\"}");
    let value = serde_json::to_value(&full).expect("request serializes");
    assert_eq!(value["appData"], json!("{\"version\":\"1.4.0\"}"));
    assert!(value.get("appDataHash").is_none());

    // Hash only -> the hash hex lives under `appData` (services `Hash` form),
    // never an `appDataHash`-only body that the orderbook rejects. This is the
    // regression lock for the latent hash-only bug.
    let mut hash_only = base();
    hash_only.app_data = QuoteAppData::hash(hash);
    let value = serde_json::to_value(&hash_only).expect("request serializes");
    assert_eq!(value["appData"], hash_wire);
    assert!(value.get("appDataHash").is_none());

    // Both -> {"appData": <document>, "appDataHash": "0x<hash>"}.
    let mut both = base();
    both.app_data = QuoteAppData::both("{\"version\":\"1.4.0\"}", hash);
    let value = serde_json::to_value(&both).expect("request serializes");
    assert_eq!(value["appData"], json!("{\"version\":\"1.4.0\"}"));
    assert_eq!(value["appDataHash"], hash_wire);

    // Neither -> both keys omitted.
    let mut empty = base();
    empty.app_data = QuoteAppData::default();
    let value = serde_json::to_value(&empty).expect("request serializes");
    assert!(value.get("appData").is_none());
    assert!(value.get("appDataHash").is_none());

    // The hash-only wire shape round-trips and resolves back to the hash.
    let roundtrip: OrderQuoteRequest =
        serde_json::from_value(serde_json::to_value(&hash_only).unwrap())
            .expect("hash-only request round-trips");
    assert_eq!(roundtrip.app_data.resolved_hash(), Some(hash));
}

#[test]
fn orders_and_trades_requests_keep_upstream_defaults() {
    let owner = sample_owner();
    let orders = OrdersQuery::new(owner);
    let trades_by_owner = TradesQuery::by_owner(owner);
    let trades_by_uid = TradesQuery::by_order_uid(sample_order_uid());

    assert_eq!(orders.offset, 0);
    assert_eq!(orders.limit, 1_000);
    assert!(trades_by_owner.is_valid());
    assert!(trades_by_uid.is_valid());
    assert_eq!(trades_by_owner.offset, 0);
    assert_eq!(trades_by_owner.limit, 10);
}

#[test]
fn order_creation_from_quote_keeps_quote_shape_and_quote_id() {
    let quote_response = serde_json::from_value::<cow_sdk_orderbook::OrderQuoteResponse>(
        sample_quote_response_json(),
    )
    .expect("quote response fixture must deserialize");
    let order = OrderCreation::from_quote(
        &quote_response,
        sample_owner(),
        None,
        SigningScheme::EthSign,
        sample_signature(),
    );

    assert_eq!(order.kind, OrderKind::Sell);
    assert_eq!(
        order.quote_id,
        Some(42),
        "from_quote must thread the response id onto the submission payload",
    );
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
fn app_data_wire_contract_is_identical_across_order_creation_and_quote_request() {
    // The signed `OrderCreation` submission DTO and the `OrderQuoteRequest`
    // quote DTO both route their `(full, hash)` app-data pair through the one
    // shared wire contract. This parity test locks "one wire contract, two
    // DTOs": for every `(full, hash)` combination the two DTOs must emit
    // identical `appData`/`appDataHash` keys, so the routing can never silently
    // diverge between a quote request and the order it backs — the exact drift
    // that hid the latent hash-only rejection in one DTO but not the other.

    let full_doc = "{\"version\":\"1.0.0\",\"metadata\":{}}";
    let hash = sample_app_data_hash();

    let cases: [(Option<&str>, Option<AppDataHash>); 4] = [
        (None, None),
        (Some(full_doc), None),
        (None, Some(hash)),
        (Some(full_doc), Some(hash)),
    ];

    for (full, app_hash) in cases {
        let mut order = OrderCreation::new(
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
        if let Some(full) = full {
            order = order.with_app_data(full);
        }
        if let Some(app_hash) = app_hash {
            order = order.with_app_data_hash(app_hash);
        }

        let mut quote = OrderQuoteRequest::new(
            sample_owner(),
            sample_buy_token(),
            sample_owner(),
            OrderQuoteSide::sell(amount("1000000")),
        );
        quote.app_data = match (full, app_hash) {
            (None, None) => QuoteAppData::default(),
            (Some(full), None) => QuoteAppData::full(full),
            (None, Some(app_hash)) => QuoteAppData::hash(app_hash),
            (Some(full), Some(app_hash)) => QuoteAppData::both(full, app_hash),
        };

        let order_value = serde_json::to_value(&order).expect("OrderCreation serializes");
        let quote_value = serde_json::to_value(&quote).expect("OrderQuoteRequest serializes");

        assert_eq!(
            order_value.get("appData"),
            quote_value.get("appData"),
            "appData must match across DTOs for (full={full:?}, hash={app_hash:?})"
        );
        assert_eq!(
            order_value.get("appDataHash"),
            quote_value.get("appDataHash"),
            "appDataHash must match across DTOs for (full={full:?}, hash={app_hash:?})"
        );
    }
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
        &quote_response,
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
