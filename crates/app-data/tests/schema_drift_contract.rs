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
//! The field-name probe is not full JSON-Schema validation.
//!
//! Two further correspondence checks anchor what the field-name probe cannot
//! see. A bounds check ties the typed numeric cap to the mirror's declared
//! `maximum`, so a refreshed mirror whose constraint changed (but whose field
//! names did not) fails until the typed bound is updated. A manifest check ties
//! the emitted [`LATEST_APP_DATA_VERSION`] and the modeled families' versions to
//! the vendored root-document manifest, so the rollup version cannot silently
//! lag the pinned upstream root.

use std::{collections::BTreeSet, fs, path::PathBuf};

use cow_sdk_app_data::{
    FlashloanHints, Hook, LATEST_APP_DATA_VERSION, PartnerFee, PartnerFeePolicy, QuoteMetadata,
};
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
    assert_schema_declares("partner-fee-v1.1.0.json", "PartnerFeePolicy", &sample);
}

/// Parses a wrapped mirror fixture into JSON for the correspondence checks.
fn read_mirror(relative: &str) -> serde_json::Value {
    serde_json::from_str(&read_schema(relative))
        .unwrap_or_else(|error| panic!("mirror {relative} must be valid JSON: {error}"))
}

/// Extracts the `vMAJOR.MINOR.PATCH` token from a schema id or `$ref` such as
/// `partnerFee/v1.1.0.json#` or `partner-fee-v1.1.0.json`.
fn schema_version_token(value: &str) -> String {
    let Some(index) = value.find("/v").or_else(|| value.find("-v")) else {
        panic!("`{value}` carries no version token");
    };
    let after = &value[index + 1..];
    let Some(end) = after.find(".json") else {
        panic!("`{value}` version token is not followed by .json");
    };
    after[..end].to_owned()
}

#[test]
fn partner_fee_volume_bound_tracks_the_mirror_maximum() {
    // The typed cap (`MAX_VOLUME_BPS`) is private; this asserts its behaviour
    // black-box against the mirror's declared `maximum`, so refreshing the
    // mirror to a new cap without updating the typed bound fails here.
    let mirror = read_mirror("partner-fee-v1.1.0.json");
    let max = mirror
        .pointer("/payload/definitions/maxVolumeBps/maximum")
        .and_then(serde_json::Value::as_u64)
        .expect("partner-fee mirror declares maxVolumeBps.maximum");
    let max = u16::try_from(max).expect("partner-fee cap must fit u16");
    let recipient = address("0x1111111111111111111111111111111111111111");

    PartnerFeePolicy::volume(max, recipient)
        .expect("the typed validator must accept the mirror maximum");
    if let Some(over) = max.checked_add(1) {
        assert!(
            PartnerFeePolicy::volume(over, recipient).is_err(),
            "the typed validator must reject mirror maximum + 1 ({over})",
        );
    }
}

#[test]
fn app_data_version_and_modeled_families_track_the_root_manifest() {
    let manifest = read_mirror("app-data-document-v1.15.0.json");

    // The version the SDK emits must equal the pinned root manifest version, so
    // an upstream root bump cannot leave the emitted version silently behind.
    let root_version = manifest
        .pointer("/payload/properties/version/default")
        .and_then(serde_json::Value::as_str)
        .expect("root manifest declares properties.version.default");
    assert_eq!(
        root_version, LATEST_APP_DATA_VERSION,
        "LATEST_APP_DATA_VERSION must equal the pinned root manifest version",
    );

    // Each modeled family we vendor a versioned mirror for must be referenced at
    // that same version by the root manifest.
    for (family_key, mirror_file) in [
        ("partnerFee", "partner-fee-v1.1.0.json"),
        ("quote", "quote-v1.1.0.json"),
    ] {
        let manifest_ref = manifest
            .pointer(&format!(
                "/payload/properties/metadata/properties/{family_key}/$ref"
            ))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("root manifest must reference {family_key}"));
        let Some(mirror_id) = read_mirror(mirror_file)
            .pointer("/payload/$id")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
        else {
            panic!("{mirror_file} must declare payload.$id");
        };
        assert_eq!(
            schema_version_token(manifest_ref),
            schema_version_token(&mirror_id),
            "{family_key}: root manifest references `{manifest_ref}` but the vendored mirror is \
             `{mirror_id}`; refresh the family mirror to the version the manifest pins",
        );
    }
}
