#![no_main]

//! Fuzz target for the typed `AppDataParams` extractor and the
//! hooks-replacement merge invariant.
//!
//! **Surface:** `cow_sdk_trading::params_from_doc` (extractor) and
//! `cow_sdk_trading::merge_and_seal_app_data` (the public seal entry that
//! drives the crate-private `merge_app_data_params` and `deep_merge_values`
//! helpers without reaching past the published API surface).
//! **Property:** `PROP-AD-002`.
//! **Seed contract:** corpus inputs cover canonical empty bases, populated
//! metadata bases, boundary signer-only and hooks-only overrides, and
//! adversarial shapes including hooks-in-override (must replace base hooks
//! fully) and hooks-in-base-with-empty-override (must preserve base hooks).
//! **Corpus README:** `../corpus/fuzz_app_data_params_from_doc/README.md`.
//!
//! The target invariants are:
//!
//! * `params_from_doc(doc).is_ok()` iff
//!   `serde_json::from_value::<AppDataParams>(doc).is_ok()` — the extractor
//!   agrees with the underlying serde contract.
//! * Hooks replacement: when the override carries a typed `hooks` field or
//!   a `metadata.hooks` entry, the merged document's `metadata.hooks` value
//!   is sourced exclusively from the override side.
//! * Function never panics on arbitrary bounded input.

use cow_sdk_app_data::{
    AppDataParams, FlashloanHints, Hook, HookList, MetadataMap, PartnerFee, PartnerFeePolicy,
};
use cow_sdk_core::{Address, Amount, AppCode, HexData};
use cow_sdk_trading::{merge_and_seal_app_data, params_from_doc};
use libfuzzer_sys::{
    arbitrary::{Arbitrary, Unstructured},
    fuzz_target,
};
use serde_json::{Map, Number, Value, json};

const MAX_STRING_BYTES: usize = 64;
const MAX_ARRAY_ITEMS: usize = 4;
const MAX_OBJECT_ITEMS: usize = 4;
const MAX_DEPTH: usize = 4;

#[derive(Debug)]
struct ParamsInput {
    base: Value,
    override_params: AppDataParams,
}

impl<'a> Arbitrary<'a> for ParamsInput {
    fn arbitrary(bytes: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        let seed_class = seed_class(read_u8(bytes, 0));
        Ok(Self {
            base: base_for_seed(seed_class, bytes),
            override_params: override_for_seed(seed_class, bytes),
        })
    }
}

fuzz_target!(|input: ParamsInput| {
    // Invariant 1: params_from_doc agrees with serde::from_value on the same
    // document. Both must classify identically on every input.
    let serde_ok = serde_json::from_value::<AppDataParams>(input.base.clone()).is_ok();
    let extractor_ok = params_from_doc(&input.base).is_ok();
    assert_eq!(
        extractor_ok, serde_ok,
        "params_from_doc must agree with serde::from_value::<AppDataParams> on the same document",
    );

    if !extractor_ok {
        // Nothing further to check on the reject path.
        return;
    }

    let base_params = params_from_doc(&input.base).expect("agreement check just proved Ok");

    // Detect whether the override carries hooks through either of the two
    // documented surfaces: the typed `hooks` slot or `metadata.hooks`.
    let override_has_typed_hooks = input.override_params.hooks.is_some();
    let override_has_metadata_hooks = input.override_params.metadata.contains_key("hooks");
    let override_has_hooks = override_has_typed_hooks || override_has_metadata_hooks;

    let Ok((info, merged_params)) = merge_and_seal_app_data(&input.base, &input.override_params)
    else {
        return;
    };

    // Idempotency carries over from the merge contract: applying the merged
    // params to its own canonical document produces byte-identical JSON.
    let (second_info, _second_params) = merge_and_seal_app_data(&info.doc, &merged_params)
        .expect("successful merged app-data must remain parseable on a second merge");
    assert_eq!(
        second_info.full_app_data, info.full_app_data,
        "typed app-data merge must be idempotent over canonical JSON",
    );

    // Invariant 2: hooks replacement. When the override supplied hooks via
    // either documented surface, the merged document's hooks value must
    // match the override-supplied hooks shape, not the base-supplied shape.
    if override_has_hooks {
        let base_hooks_value = base_hooks_metadata_value(&base_params);
        let override_hooks_value = override_hooks_metadata_value(&input.override_params);
        let merged_hooks_value = base_hooks_metadata_value(&merged_params);

        // The base side's hooks must not silently survive when the override
        // supplied any hooks shape (replacement, not deep merge).
        if let Some(base_hooks) = &base_hooks_value {
            if Some(base_hooks) != override_hooks_value.as_ref() {
                assert_ne!(
                    merged_hooks_value.as_ref(),
                    Some(base_hooks),
                    "override-supplied hooks must replace base-supplied hooks instead of falling through to the base value",
                );
            }
        }
    }
});

/// Returns the `metadata.hooks` value for the given typed params, drawing
/// from both the typed `hooks` slot and the open-ended `metadata` map.
fn base_hooks_metadata_value(params: &AppDataParams) -> Option<Value> {
    if let Some(hooks) = &params.hooks {
        return Some(serde_json::to_value(hooks).expect("typed HookList must serialize"));
    }
    params.metadata.get("hooks").cloned()
}

/// Returns the override-side `metadata.hooks` view used by the assertion.
fn override_hooks_metadata_value(params: &AppDataParams) -> Option<Value> {
    if let Some(hooks) = &params.hooks {
        return Some(serde_json::to_value(hooks).expect("typed HookList must serialize"));
    }
    params.metadata.get("hooks").cloned()
}

fn base_for_seed(seed_class: u8, bytes: &mut Unstructured<'_>) -> Value {
    match seed_class {
        0 => Value::Object(Map::new()),
        1 => json!({
            "appCode": "CoW Swap",
            "metadata": {
                "quote": { "slippageBips": 50 },
                "orderClass": { "orderClass": "market" }
            },
            "version": "1.14.0"
        }),
        2 => json!({
            "appCode": "CoW Swap",
            "environment": "production",
            "metadata": {
                "signer": address_from_seed(0x11),
                "partnerFee": partner_fee_value(false),
                "hooks": hooks_value(0x33),
                "flashloan": flashloan_value(0x44)
            },
            "version": "1.14.0"
        }),
        3 => json!({
            "appCode": "CoW Swap",
            "metadata": { "hooks": hooks_value(0x22) },
            "version": "1.14.0"
        }),
        _ => bounded_json(bytes, 0),
    }
}

fn override_for_seed(seed_class: u8, bytes: &mut Unstructured<'_>) -> AppDataParams {
    match seed_class {
        0 => AppDataParams::default(),
        1 => AppDataParams::default().with_metadata(metadata_from_value(json!({
            "quote": { "slippageBips": 50 }
        }))),
        2 => AppDataParams::default().with_hooks(hooks(0x55)),
        3 => AppDataParams::default().with_metadata(metadata_from_value(json!({
            "hooks": hooks_value(0x66)
        }))),
        4 => AppDataParams::default().with_signer(address(0x77)),
        _ => {
            let mut params = AppDataParams::default()
                .with_metadata(metadata_from_value(bounded_json(bytes, 0)));
            params.app_code = maybe_string(bytes, "CoW Swap").and_then(|s| AppCode::new(s).ok());
            params.environment = maybe_string(bytes, "production");
            params.signer =
                read_bool(bytes, false).then(|| address_from_bytes(read_address(bytes, 0x77)));
            params.flashloan = read_bool(bytes, false).then(|| flashloan(read_u8(bytes, 0x88)));
            params
        }
    }
}

fn bounded_json(bytes: &mut Unstructured<'_>, depth: usize) -> Value {
    if depth >= MAX_DEPTH {
        return scalar_json(bytes);
    }

    match read_u8(bytes, 0) % 6 {
        0 => Value::Null,
        1 => Value::Bool(read_bool(bytes, false)),
        2 => number_json(read_u64(bytes, 0)),
        3 => Value::String(read_string(bytes, MAX_STRING_BYTES)),
        4 => {
            let len = usize::from(read_u8(bytes, 0)) % (MAX_ARRAY_ITEMS + 1);
            Value::Array(
                (0..len)
                    .map(|_| bounded_json(bytes, depth + 1))
                    .collect::<Vec<_>>(),
            )
        }
        _ => {
            let len = usize::from(read_u8(bytes, 0)) % (MAX_OBJECT_ITEMS + 1);
            let mut object = Map::new();
            for index in 0..len {
                object.insert(object_key(bytes, index), bounded_json(bytes, depth + 1));
            }
            Value::Object(object)
        }
    }
}

fn scalar_json(bytes: &mut Unstructured<'_>) -> Value {
    match read_u8(bytes, 0) % 4 {
        0 => Value::Null,
        1 => Value::Bool(read_bool(bytes, false)),
        2 => number_json(read_u64(bytes, 0)),
        _ => Value::String(read_string(bytes, MAX_STRING_BYTES)),
    }
}

fn number_json(value: u64) -> Value {
    Value::Number(Number::from(value))
}

fn object_key(bytes: &mut Unstructured<'_>, index: usize) -> String {
    const KEYS: &[&str] = &[
        "metadata",
        "quote",
        "orderClass",
        "signer",
        "partnerFee",
        "hooks",
        "flashloan",
        "value",
        "shared",
    ];
    let selected = usize::from(read_u8(bytes, index as u8)) % KEYS.len();
    KEYS[selected].to_owned()
}

fn metadata_from_value(value: Value) -> MetadataMap {
    match value {
        Value::Object(map) => map,
        other => {
            let mut map = Map::new();
            map.insert("fuzz".to_owned(), other);
            map
        }
    }
}

fn maybe_string(bytes: &mut Unstructured<'_>, fallback: &str) -> Option<String> {
    read_bool(bytes, false).then(|| {
        let value = read_string(bytes, MAX_STRING_BYTES);
        if value.is_empty() {
            fallback.to_owned()
        } else {
            value
        }
    })
}

fn read_string(bytes: &mut Unstructured<'_>, max_len: usize) -> String {
    let len = usize::from(read_u8(bytes, 0)) % (max_len + 1);
    let mut raw = vec![0u8; len];
    for byte in &mut raw {
        *byte = read_u8(bytes, b'a');
    }
    String::from_utf8_lossy(&raw).into_owned()
}

fn seed_class(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        _ => value % 6,
    }
}

fn read_u8(bytes: &mut Unstructured<'_>, default: u8) -> u8 {
    u8::arbitrary(bytes).unwrap_or(default)
}

fn read_bool(bytes: &mut Unstructured<'_>, default: bool) -> bool {
    bool::arbitrary(bytes).unwrap_or(default)
}

fn read_u64(bytes: &mut Unstructured<'_>, default: u64) -> u64 {
    u64::arbitrary(bytes).unwrap_or(default)
}

fn read_address(bytes: &mut Unstructured<'_>, fallback: u8) -> [u8; 20] {
    <[u8; 20]>::arbitrary(bytes).unwrap_or([fallback; 20])
}

fn address(seed: u8) -> Address {
    address_from_bytes([seed; 20])
}

fn address_from_seed(seed: u8) -> String {
    address(seed).to_string()
}

fn address_from_bytes(bytes: [u8; 20]) -> Address {
    Address::from_bytes(bytes)
}

fn amount(value: u128) -> Amount {
    Amount::new(value.to_string()).expect("u128 string must remain a valid amount")
}

fn partner_fee_value(multiple: bool) -> Value {
    let recipient = address(0xaa);
    let first = PartnerFeePolicy::volume(42, recipient.clone())
        .expect("fixture volume partner fee must validate");
    let value = if multiple {
        PartnerFee::from(vec![
            first,
            PartnerFeePolicy::surplus(250, 100, recipient)
                .expect("fixture surplus partner fee must validate"),
        ])
    } else {
        PartnerFee::from(first)
    };
    value.to_value()
}

fn hooks(seed: u8) -> HookList {
    HookList::new(
        Vec::new(),
        vec![Hook::new(
            address(seed),
            HexData::new("0x02").expect("fixture hook calldata must be valid"),
            100_000,
        )],
    )
    .with_version("0.2.0")
}

fn hooks_value(seed: u8) -> Value {
    serde_json::to_value(hooks(seed)).expect("fixture hooks must serialize")
}

fn flashloan(seed: u8) -> FlashloanHints {
    FlashloanHints::new(
        address(seed.max(1)),
        address(seed.wrapping_add(1).max(1)),
        address(seed.wrapping_add(2).max(1)),
        address(seed.wrapping_add(3).max(1)),
        amount(2_000_000_000_000_000_000),
    )
    .expect("fixture flash-loan hint must validate")
}

fn flashloan_value(seed: u8) -> Value {
    serde_json::to_value(flashloan(seed)).expect("fixture flashloan must serialize")
}
