//! Schema drift gate for the typed app-data metadata surface.
//!
//! Runtime validation is performed by the typed metadata structs, not by a
//! JSON-Schema validator. One self-contained drift fixture per modeled metadata
//! family lives under `parity/fixtures/app_data/schemas/` (with lock-validated
//! provenance headers), mirroring the upstream JSON Schema's field-name set at
//! the pinned commit. Each test serializes a typed sample of the modeled struct
//! and asserts every wire field name it emits is declared in the schema mirror,
//! so renaming a field's serde name on the Rust struct — or refreshing the
//! mirror to an upstream that dropped a field — fails here at review time
//! instead of silently diverging. Samples are populated minimally so only the
//! fields the schema is expected to declare appear; optional forward-compat
//! fields the schema does not carry (e.g. `QuoteMetadata.version`) stay unset.
//! The probe is a field-name correspondence check, not full JSON-Schema
//! validation.

use std::{collections::BTreeSet, fs, path::PathBuf};

use cow_sdk_app_data::{FlashloanHints, Hook, PartnerFee, PartnerFeePolicy, QuoteMetadata};
use cow_sdk_core::{Address, Amount, HexData};
use serde::Serialize;

fn read_schema(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../parity/fixtures/app_data/schemas")
        .join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "schema fixture {} must be readable: {error}",
            path.display()
        )
    })
}

fn address(hex: &str) -> Address {
    Address::new(hex).expect("test address literal must be valid")
}

/// Collects every object key emitted anywhere in `value`.
fn emitted_field_names(value: &serde_json::Value, out: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                out.insert(key.clone());
                emitted_field_names(nested, out);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                emitted_field_names(item, out);
            }
        }
        _ => {}
    }
}

/// Serializes `sample` and asserts the schema mirror declares every wire field
/// name it emits.
fn assert_schema_declares<T: Serialize>(relative: &str, typed_surface: &str, sample: &T) {
    let body = read_schema(relative);
    let value = serde_json::to_value(sample).expect("typed metadata sample must serialize");
    let mut names = BTreeSet::new();
    emitted_field_names(&value, &mut names);
    assert!(
        !names.is_empty(),
        "{typed_surface} sample must emit at least one field",
    );
    for name in &names {
        assert!(
            body.contains(name.as_str()),
            "{relative} no longer declares `{name}` emitted by {typed_surface}; the schema mirror \
             and the typed struct have diverged",
        );
    }
}

#[test]
fn flashloan_schema_matches_the_typed_flashloan_hint() {
    let sample = FlashloanHints::new(
        address("0x1111111111111111111111111111111111111111"),
        address("0x2222222222222222222222222222222222222222"),
        address("0x3333333333333333333333333333333333333333"),
        address("0x4444444444444444444444444444444444444444"),
        Amount::new("1000000000000000000").expect("amount literal must be valid"),
    )
    .expect("flashloan sample must validate");
    assert_schema_declares("flashloan.json", "FlashloanHints", &sample);
}

#[test]
fn quote_schema_matches_the_typed_quote_metadata() {
    let mut sample = QuoteMetadata::new(50).expect("slippage literal must validate");
    // Exercise the optional `smartSlippage` field too — it is part of the
    // schema; `version` is intentionally left unset (it is an SDK-side
    // forward-compat field the v1.1.0 schema does not declare).
    sample.smart_slippage = Some(true);
    assert_schema_declares("quote-v1.1.0.json", "QuoteMetadata", &sample);
}

#[test]
fn hook_schema_matches_the_typed_hook() {
    let mut sample = Hook::new(
        address("0x1234567890abcdef1234567890abcdef12345678"),
        HexData::new("0x01020304").expect("hook calldata literal must be valid"),
        100_000,
    );
    sample.dapp_id = Some("cow-rs".to_owned());
    assert_schema_declares("hook-v0.2.0.json", "Hook", &sample);
}

#[test]
fn partner_fee_schema_matches_the_typed_policy_shape() {
    let recipient = address("0x1111111111111111111111111111111111111111");
    // Exercise all three policy variants so every policy field name the schema
    // declares (volumeBps / surplusBps / priceImprovementBps / maxVolumeBps /
    // recipient) is checked against the mirror.
    let sample = PartnerFee::from(vec![
        PartnerFeePolicy::volume(42, recipient).expect("volume policy must validate"),
        PartnerFeePolicy::surplus(250, 100, recipient).expect("surplus policy must validate"),
        PartnerFeePolicy::price_improvement(250, 100, recipient)
            .expect("price-improvement policy must validate"),
    ]);
    assert_schema_declares("partner-fee-v1.0.0.json", "PartnerFeePolicy", &sample);
}
