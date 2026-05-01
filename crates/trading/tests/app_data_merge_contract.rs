//! Contract suite pinning the typed app-data merge pipeline consumed by
//! the quote-to-post submission path.
//!
//! The typed pipeline re-deserializes the sealed quote-derived wire
//! document through the existing [`cow_sdk_app_data::AppDataParams`]
//! impl, merges the base and override as typed values through
//! [`cow_sdk_trading::merge_and_seal_app_data`], and re-emits the
//! canonical wire document through the existing
//! [`cow_sdk_app_data::generate_app_data_doc`] /
//! [`cow_sdk_app_data::get_app_data_info`] helpers. The pinned
//! invariants below lock that typed pipeline to the reviewed upstream
//! merge semantics so drift in either the typed seam or the downstream
//! submission validator surfaces through a failing test before it
//! reaches release.

#![allow(
    clippy::doc_markdown,
    clippy::option_if_let_else,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "pedantic lint group acceptable inside integration test code"
)]

mod common;

use cow_sdk_app_data::{
    AppDataParams, FlashloanHints, Hook, HookList, MetadataMap, PartnerFee, PartnerFeePolicy,
    generate_app_data_doc, get_app_data_info,
};
use cow_sdk_core::{Amount, HexData, OrderKind};
use cow_sdk_trading::{
    ClientRejection, SwapAdvancedSettings, TradingError, get_quote_results,
    merge_and_seal_app_data, params_from_doc, post_swap_order_from_quote,
};
use serde_json::{Value, json};

use crate::common::{
    ALT_RECEIVER, MockOrderbook, MockSigner, OWNER, address, sample_trade_parameters,
    sample_trader_parameters, sell_quote_response,
};

const FLASH_LIQUIDITY_PROVIDER: &str = "0xb50201558B00496A145fE76f7424749556E326D8";
const FLASH_PROTOCOL_ADAPTER: &str = "0x1186B5ad42E3e6d6c6901FC53b4A367540E6EcFE";
const FLASH_RECEIVER: &str = "0x1186B5ad42E3e6d6c6901FC53b4A367540E6EcFE";
const FLASH_TOKEN: &str = "0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d";
const FLASH_AMOUNT: &str = "2000000000000000000";
const HOOK_TARGET_PRE: &str = "0x1234567890abcdef1234567890abcdef12345678";
const HOOK_TARGET_POST: &str = "0xabcdef1234567890abcdef1234567890abcdef12";
const HOOK_GAS_LIMIT: &str = "100000";
const THIRD_OWNER: &str = "0x3333333333333333333333333333333333333333";

fn sample_flashloan() -> FlashloanHints {
    FlashloanHints::new(
        address(FLASH_LIQUIDITY_PROVIDER),
        address(FLASH_PROTOCOL_ADAPTER),
        address(FLASH_RECEIVER),
        address(FLASH_TOKEN),
        Amount::new(FLASH_AMOUNT).expect("fixture flash-loan amount must be valid"),
    )
    .expect("fixture flash-loan hints must validate")
}

fn base_params_with_quote_metadata() -> AppDataParams {
    let metadata: MetadataMap = serde_json::from_value(json!({
        "quote": { "slippageBips": 50 },
        "orderClass": { "orderClass": "market" }
    }))
    .expect("base metadata fixture must build");
    AppDataParams::new(
        Some("CoW Swap".to_owned()),
        Some("production".to_owned()),
        None,
        None,
        metadata,
    )
}

fn hooks_pre_value() -> Value {
    json!({
        "version": "0.2.0",
        "pre": [{
            "target": HOOK_TARGET_PRE,
            "callData": "0x01",
            "gasLimit": HOOK_GAS_LIMIT,
        }]
    })
}

fn hooks_post_value() -> Value {
    json!({
        "version": "0.2.0",
        "post": [{
            "target": HOOK_TARGET_POST,
            "callData": "0x02",
            "gasLimit": HOOK_GAS_LIMIT,
        }]
    })
}

fn typed_post_hooks() -> HookList {
    HookList::new(
        Vec::new(),
        vec![Hook::new(
            address(HOOK_TARGET_POST),
            HexData::new("0x02").expect("fixture hook calldata must be valid"),
            HOOK_GAS_LIMIT
                .parse::<u64>()
                .expect("fixture gas limit must be valid"),
        )],
    )
    .with_version("0.2.0")
}

fn sealed_base_doc(params: AppDataParams) -> Value {
    let doc = generate_app_data_doc(params);
    // Exercise the full seal pipeline so test inputs mirror the
    // quote-produced wire document byte-identically.
    let _ = get_app_data_info(doc.clone()).expect("sealed base doc must pass app-data validation");
    doc
}

#[test]
fn override_with_only_signer_survives_into_wire_doc() {
    let base_doc = sealed_base_doc(base_params_with_quote_metadata());
    let signer = address(OWNER);

    let override_params = AppDataParams::default().with_signer(signer.clone());

    let (info, merged_params) = merge_and_seal_app_data(&base_doc, &override_params)
        .expect("typed merge must succeed with signer-only override");

    assert_eq!(
        merged_params.signer,
        Some(signer.clone()),
        "override signer must survive the typed merge"
    );
    assert_eq!(
        info.doc["metadata"]["signer"].as_str(),
        Some(signer.as_str()),
        "wire doc must carry metadata.signer lifted from the typed field"
    );
    assert_eq!(
        info.doc["metadata"]["quote"], base_doc["metadata"]["quote"],
        "unrelated base metadata keys must be preserved byte-identical"
    );
    assert_eq!(
        info.doc["metadata"]["orderClass"], base_doc["metadata"]["orderClass"],
        "base order class metadata must be preserved byte-identical"
    );
    assert_eq!(
        info.doc["appCode"], base_doc["appCode"],
        "top-level appCode must be preserved when not overridden"
    );
    assert_eq!(
        info.doc["environment"], base_doc["environment"],
        "top-level environment must be preserved when not overridden"
    );
}

#[test]
fn merge_preserves_override_signer_byte_identical() {
    let base_doc = sealed_base_doc(base_params_with_quote_metadata());
    let signer = address(OWNER);
    let override_params = AppDataParams::default().with_signer(signer.clone());

    let (info, merged_params) = merge_and_seal_app_data(&base_doc, &override_params)
        .expect("typed merge with signer override must succeed");

    assert_eq!(merged_params.signer, Some(signer.clone()));
    assert_eq!(
        info.doc["metadata"]["signer"].as_str(),
        Some(signer.as_str()),
        "override signer must be carried to the wire byte-identical",
    );
}

#[test]
fn merge_replaces_hooks_per_adr_0018() {
    let mut base_metadata = base_params_with_quote_metadata().metadata;
    base_metadata.insert("hooks".to_owned(), hooks_pre_value());
    let base_doc = sealed_base_doc(base_params_with_quote_metadata().with_metadata(base_metadata));
    let override_params = AppDataParams::default().with_hooks(typed_post_hooks());

    let (info, _merged_params) = merge_and_seal_app_data(&base_doc, &override_params)
        .expect("typed hooks override must succeed");

    assert!(
        info.doc["metadata"]["hooks"].get("pre").is_none(),
        "override hooks must replace the base hooks envelope",
    );
    assert_eq!(
        info.doc["metadata"]["hooks"]["post"],
        hooks_post_value()["post"],
        "override post hooks must become the final wire hook set",
    );
}

#[test]
fn merge_lifts_flashloan_metadata_through_quote_to_post() {
    let base_doc = sealed_base_doc(base_params_with_quote_metadata());
    let hints = sample_flashloan();
    let override_params = AppDataParams::default().with_flashloan(hints.clone());

    let (info, merged_params) = merge_and_seal_app_data(&base_doc, &override_params)
        .expect("typed flashloan override must succeed");
    let expected = serde_json::to_value(&hints).expect("flashloan hints must serialize");

    assert_eq!(merged_params.flashloan, Some(hints));
    assert_eq!(
        info.doc["metadata"]["flashloan"], expected,
        "typed flashloan metadata must be lifted into the final wire document",
    );
}

#[test]
fn override_with_only_flashloan_survives_into_wire_doc() {
    let base_doc = sealed_base_doc(base_params_with_quote_metadata());
    let hints = sample_flashloan();

    let override_params = AppDataParams::default().with_flashloan(hints.clone());

    let (info, merged_params) = merge_and_seal_app_data(&base_doc, &override_params)
        .expect("typed merge must succeed with flashloan-only override");

    assert_eq!(
        merged_params.flashloan,
        Some(hints.clone()),
        "override flash-loan hints must survive the typed merge"
    );
    let flashloan_value =
        serde_json::to_value(&hints).expect("flash-loan hints must reserialize through serde");
    assert_eq!(
        info.doc["metadata"]["flashloan"], flashloan_value,
        "wire doc must carry metadata.flashloan lifted from the typed field"
    );
    assert_eq!(
        info.doc["metadata"]["quote"], base_doc["metadata"]["quote"],
        "unrelated base metadata must remain byte-identical"
    );
}

#[test]
fn override_with_both_signer_and_flashloan_survives() {
    let base_doc = sealed_base_doc(base_params_with_quote_metadata());
    let signer = address(OWNER);
    let hints = sample_flashloan();

    let override_params = AppDataParams::default()
        .with_signer(signer.clone())
        .with_flashloan(hints.clone());

    let (info, merged_params) = merge_and_seal_app_data(&base_doc, &override_params)
        .expect("typed merge must succeed with signer and flashloan override");

    assert_eq!(merged_params.signer, Some(signer.clone()));
    assert_eq!(merged_params.flashloan, Some(hints.clone()));

    let flashloan_value =
        serde_json::to_value(&hints).expect("flash-loan hints must reserialize through serde");
    assert_eq!(
        info.doc["metadata"]["signer"].as_str(),
        Some(signer.as_str()),
        "wire doc must carry the typed signer field",
    );
    assert_eq!(
        info.doc["metadata"]["flashloan"], flashloan_value,
        "wire doc must carry the typed flash-loan hints",
    );
}

#[test]
fn base_hooks_cleared_when_override_contains_hooks() {
    let mut base_metadata = base_params_with_quote_metadata().metadata;
    base_metadata.insert("hooks".to_owned(), hooks_pre_value());
    let base = base_params_with_quote_metadata().with_metadata(base_metadata);
    let base_doc = sealed_base_doc(base);

    let mut override_metadata = MetadataMap::new();
    override_metadata.insert("hooks".to_owned(), hooks_post_value());
    let override_params = AppDataParams::default().with_metadata(override_metadata);

    let (info, _merged_params) = merge_and_seal_app_data(&base_doc, &override_params)
        .expect("typed merge must succeed with hooks replacement");

    let hooks = info
        .doc
        .get("metadata")
        .and_then(|metadata| metadata.get("hooks"))
        .expect("merged wire doc must carry metadata.hooks");

    assert!(
        hooks.get("pre").is_none(),
        "base-side metadata.hooks.pre must be dropped when the override supplies hooks",
    );
    assert_eq!(
        hooks.get("post"),
        hooks_post_value().get("post"),
        "override metadata.hooks.post must be preserved byte-identical",
    );
    // Non-hooks metadata keys on the base side must still survive.
    assert_eq!(
        info.doc["metadata"]["quote"], base_doc["metadata"]["quote"],
        "unrelated base metadata keys must remain byte-identical after hooks replacement",
    );
}

#[test]
fn base_hooks_preserved_when_override_has_no_hooks() {
    let mut base_metadata = base_params_with_quote_metadata().metadata;
    base_metadata.insert("hooks".to_owned(), hooks_pre_value());
    let base = base_params_with_quote_metadata().with_metadata(base_metadata);
    let base_doc = sealed_base_doc(base);

    let override_params = AppDataParams::default();

    let (info, _merged_params) = merge_and_seal_app_data(&base_doc, &override_params)
        .expect("typed merge must succeed when the override omits hooks");

    let hooks = info
        .doc
        .get("metadata")
        .and_then(|metadata| metadata.get("hooks"))
        .expect("merged wire doc must carry metadata.hooks when the base supplies it");

    assert_eq!(
        hooks,
        &hooks_pre_value(),
        "base metadata.hooks must survive byte-identical when the override has no hooks key",
    );
}

#[test]
fn typed_hooks_override_replaces_base_hooks_and_survives_merge() {
    let mut base_metadata = base_params_with_quote_metadata().metadata;
    base_metadata.insert("hooks".to_owned(), hooks_pre_value());
    let base = base_params_with_quote_metadata().with_metadata(base_metadata);
    let base_doc = sealed_base_doc(base);

    let hooks = typed_post_hooks();
    let override_params = AppDataParams::default().with_hooks(hooks.clone());

    let (info, merged_params) = merge_and_seal_app_data(&base_doc, &override_params)
        .expect("typed merge must succeed with typed hooks replacement");

    assert_eq!(
        merged_params.hooks,
        Some(hooks.clone()),
        "typed hooks must survive the app-data merge result",
    );

    let hooks_value = serde_json::to_value(&hooks).expect("typed hooks must serialize");
    assert_eq!(
        info.doc["metadata"]["hooks"], hooks_value,
        "wire doc must carry the typed hooks override byte-identical",
    );
    assert!(
        info.doc["metadata"]["hooks"].get("pre").is_none(),
        "base hook pre-array must be dropped when typed override hooks are supplied",
    );
}

#[tokio::test]
async fn base_doc_signer_triggers_appdata_from_mismatch_when_from_differs() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let base_signer = address(OWNER);
    let submission_owner = address(ALT_RECEIVER);
    let signer = MockSigner::new(submission_owner.clone());
    let mut trade = sample_trade_parameters(OrderKind::Sell);
    trade.owner = Some(submission_owner.clone());

    let advanced_at_quote = SwapAdvancedSettings::new()
        .with_app_data(AppDataParams::default().with_signer(base_signer.clone()));

    let quote_results = get_quote_results(
        &trade,
        &trader,
        &signer,
        Some(&advanced_at_quote),
        &orderbook,
    )
    .await
    .expect("quote with typed signer must succeed");

    let error = post_swap_order_from_quote(&quote_results, &trader, &signer, None, &orderbook)
        .await
        .expect_err("appdata signer on the base doc must reject when from diverges");

    match error {
        TradingError::ClientRejected(ClientRejection::AppdataFromMismatch {
            appdata_signer,
            from,
        }) => {
            assert_eq!(
                appdata_signer, base_signer,
                "typed rejection must surface the base-doc signer"
            );
            assert_eq!(
                from, submission_owner,
                "typed rejection must surface the submission-time from address"
            );
        }
        other => panic!("expected AppdataFromMismatch, got {other:?}"),
    }
    assert!(
        orderbook.state().sent_orders.is_empty(),
        "order submission must not fire when the typed validator rejects",
    );
}

#[tokio::test]
async fn partner_fee_in_advanced_settings_appdata_merges_through_to_post() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let signer = MockSigner::default();
    let trade = sample_trade_parameters(OrderKind::Sell);
    let partner_fee = PartnerFee::from(
        PartnerFeePolicy::volume(42, address(ALT_RECEIVER))
            .expect("partner-fee fixture must validate"),
    )
    .to_value();
    let advanced = SwapAdvancedSettings::new().with_app_data(
        AppDataParams::default().with_metadata(
            serde_json::from_value(json!({
                "partnerFee": partner_fee,
            }))
            .expect("advanced app-data metadata must deserialize"),
        ),
    );

    let quote_results = get_quote_results(&trade, &trader, &signer, Some(&advanced), &orderbook)
        .await
        .expect("quote with partner-fee app-data override must succeed");
    post_swap_order_from_quote(
        &quote_results,
        &trader,
        &signer,
        Some(&advanced),
        &orderbook,
    )
    .await
    .expect("quote-derived post must preserve partner-fee app-data override");
    let sent = orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .expect("post path must submit an order");
    let app_data: Value = serde_json::from_str(
        sent.app_data
            .as_deref()
            .expect("posted order must carry full app-data"),
    )
    .expect("posted full app-data must remain valid json");

    assert_eq!(
        app_data["metadata"]["partnerFee"]["volumeBps"],
        json!(42),
        "advancedSettings.appData partner fee must merge through to the posted order body",
    );
}

#[tokio::test]
async fn base_doc_signer_matches_from_passes_validation() {
    let trader = sample_trader_parameters();
    let orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let base_signer = address(OWNER);
    let signer = MockSigner::new(base_signer.clone());
    let mut trade = sample_trade_parameters(OrderKind::Sell);
    trade.owner = Some(base_signer.clone());

    let advanced_at_quote = SwapAdvancedSettings::new()
        .with_app_data(AppDataParams::default().with_signer(base_signer.clone()));

    let quote_results = get_quote_results(
        &trade,
        &trader,
        &signer,
        Some(&advanced_at_quote),
        &orderbook,
    )
    .await
    .expect("quote with matching signer must succeed");

    let result = post_swap_order_from_quote(&quote_results, &trader, &signer, None, &orderbook)
        .await
        .expect("matching base-doc signer and from must pass the typed validator");

    assert_eq!(result.order_id, crate::common::order_uid());
    assert_eq!(orderbook.state().sent_orders.len(), 1);
}

#[tokio::test]
async fn override_signer_supersedes_base_signer() {
    // Sub-case A: override replaces base signer with C, submission from is C → accepted.
    let trader = sample_trader_parameters();
    let orderbook_a = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let override_signer = address(THIRD_OWNER);
    let signer_c = MockSigner::new(override_signer.clone());
    let mut trade_c = sample_trade_parameters(OrderKind::Sell);
    trade_c.owner = Some(override_signer.clone());

    let advanced_at_quote = SwapAdvancedSettings::new()
        .with_app_data(AppDataParams::default().with_signer(address(OWNER)));
    let advanced_at_post = SwapAdvancedSettings::new()
        .with_app_data(AppDataParams::default().with_signer(override_signer.clone()));

    let quote_results_c = get_quote_results(
        &trade_c,
        &trader,
        &signer_c,
        Some(&advanced_at_quote),
        &orderbook_a,
    )
    .await
    .expect("quote with base signer A must succeed");
    post_swap_order_from_quote(
        &quote_results_c,
        &trader,
        &signer_c,
        Some(&advanced_at_post),
        &orderbook_a,
    )
    .await
    .expect("override signer must supersede the base-doc signer when from matches the override");

    // Sub-case B: same override signer C, submission from is A → rejected with
    // AppdataFromMismatch surfacing the override signer.
    let orderbook_b = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let base_signer = address(OWNER);
    let signer_a = MockSigner::new(base_signer.clone());
    let mut trade_a = sample_trade_parameters(OrderKind::Sell);
    trade_a.owner = Some(base_signer.clone());

    let quote_results_a = get_quote_results(
        &trade_a,
        &trader,
        &signer_a,
        Some(&advanced_at_quote),
        &orderbook_b,
    )
    .await
    .expect("quote with base signer A must succeed for sub-case B");

    let error = post_swap_order_from_quote(
        &quote_results_a,
        &trader,
        &signer_a,
        Some(&advanced_at_post),
        &orderbook_b,
    )
    .await
    .expect_err("override signer must reject a diverging submission from");

    match error {
        TradingError::ClientRejected(ClientRejection::AppdataFromMismatch {
            appdata_signer,
            from,
        }) => {
            assert_eq!(
                appdata_signer, override_signer,
                "typed rejection must surface the override signer that the wire doc carried",
            );
            assert_eq!(
                from, base_signer,
                "typed rejection must surface the submission-time from address",
            );
        }
        other => panic!("expected AppdataFromMismatch, got {other:?}"),
    }
}

#[test]
fn round_trip_idempotency() {
    let signer = address(OWNER);
    let hints = sample_flashloan();
    let mut metadata: MetadataMap = serde_json::from_value(json!({
        "quote": { "slippageBips": 50 },
        "orderClass": { "orderClass": "market" },
    }))
    .expect("round-trip fixture metadata must build");
    metadata.insert(
        "utm".to_owned(),
        json!({ "utmSource": "cow-rs", "utmMedium": "test" }),
    );
    let original = AppDataParams::new(
        Some("CoW Swap".to_owned()),
        Some("production".to_owned()),
        Some(signer),
        Some(hints),
        metadata,
    );

    let doc = generate_app_data_doc(original.clone());
    let recovered = params_from_doc(&doc).expect("round-trip through params_from_doc must succeed");

    assert_eq!(
        recovered, original,
        "generate_app_data_doc + params_from_doc must be idempotent across the typed surface",
    );
}

#[test]
fn user_consents_array_replaced_not_concatenated() {
    // Documents the Rust-side behavior: arrays replace (rather than
    // concatenate) under the typed merge pipeline, so the explicit
    // `userConsents` replacement rule the reviewed upstream SDK applies
    // manually is already covered by Rust's deep-merge semantics and
    // does not require a dedicated carve-out.
    let mut base_metadata = base_params_with_quote_metadata().metadata;
    base_metadata.insert(
        "userConsents".to_owned(),
        json!([
            { "terms": "QmBaseTermsHashAAAAAAAAAAAAAAAAAAAAAAAAAAAA", "acceptedDate": "2025-11-11T23:00:00Z" },
            { "terms": "QmBaseTermsHashBBBBBBBBBBBBBBBBBBBBBBBBBBBB", "acceptedDate": "2025-11-11T23:10:00Z" },
        ]),
    );
    let base = base_params_with_quote_metadata().with_metadata(base_metadata);
    let base_doc = sealed_base_doc(base);

    let mut override_metadata = MetadataMap::new();
    override_metadata.insert(
        "userConsents".to_owned(),
        json!([
            { "terms": "QmOverrideTermsHashCCCCCCCCCCCCCCCCCCCCCCCC", "acceptedDate": "2025-11-12T08:30:00Z" },
        ]),
    );
    let override_params = AppDataParams::default().with_metadata(override_metadata);

    let (info, _merged_params) = merge_and_seal_app_data(&base_doc, &override_params)
        .expect("typed merge must succeed with userConsents replacement");

    let merged_consents = info
        .doc
        .get("metadata")
        .and_then(|metadata| metadata.get("userConsents"))
        .and_then(Value::as_array)
        .expect("merged wire doc must carry metadata.userConsents as an array");

    assert_eq!(
        merged_consents.len(),
        1,
        "user-consents arrays must be replaced rather than concatenated under the typed merge",
    );
    assert_eq!(
        merged_consents[0],
        json!({
            "terms": "QmOverrideTermsHashCCCCCCCCCCCCCCCCCCCCCCCC",
            "acceptedDate": "2025-11-12T08:30:00Z",
        }),
        "merged metadata.userConsents must equal the override array byte-identical",
    );
}
