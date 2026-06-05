#![no_main]

//! Fuzz target for the typed app-data merge pipeline.
//!
//! **Surface:** `cow_sdk_trading::merge_and_seal_app_data`.
//! **Property:** `PROP-AD-002`.
//! **Seed contract:** corpus inputs cover empty and nested bases, populated
//! metadata, signer conflicts, partner-fee single/multiple shapes, hooks
//! replacement, flash-loan metadata, and key collisions.
//!
//! The target maps arbitrary bytes into a bounded `(serde_json::Value,
//! AppDataParams)` pair, attempts the typed merge, and asserts that any
//! successful merge is idempotent: applying the merged typed params to the
//! canonical document produced by the first merge yields byte-identical
//! canonical JSON.

use cow_sdk_app_data::{
    AppDataParams, FlashloanHints, Hook, HookList, MetadataMap, PartnerFee, PartnerFeePolicy,
};
use cow_sdk_core::{Address, Amount, AppCode, HexData};
use cow_sdk_trading::merge_and_seal_app_data;
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
struct MergeInput {
    base: Value,
    override_params: AppDataParams,
}

impl<'a> Arbitrary<'a> for MergeInput {
    fn arbitrary(bytes: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        let seed_class = seed_class(read_u8(bytes, 0));
        Ok(Self {
            base: base_for_seed(seed_class, bytes),
            override_params: override_for_seed(seed_class, bytes),
        })
    }
}

fuzz_target!(|input: MergeInput| {
    let Ok((info, merged_params)) = merge_and_seal_app_data(&input.base, &input.override_params)
    else {
        return;
    };

    let (second_info, _second_params) = merge_and_seal_app_data(&info.doc, &merged_params)
        .expect("successful merged app-data must remain parseable on a second merge");
    assert_eq!(
        second_info.full_app_data, info.full_app_data,
        "typed app-data merge must be idempotent over canonical JSON",
    );
});

fn base_for_seed(seed_class: u8, bytes: &mut Unstructured<'_>) -> Value {
    match seed_class {
        0 => Value::Object(Map::new()),
        1 => json!({
            "appCode": "CoW Swap",
            "metadata": {
                "quote": { "slippageBips": 50, "nested": { "a": { "b": { "c": true } } } },
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
                "flashloan": flashloan_value(0x44),
                "userConsents": [{ "terms": "QmBaseTermsHash", "acceptedDate": "2025-11-11T23:00:00Z" }]
            },
            "version": "1.14.0"
        }),
        3 => json!({
            "appCode": "CoW Swap",
            "metadata": { "signer": address_from_seed(0x11), "quote": { "slippageBips": 42 } },
            "version": "1.14.0"
        }),
        8 => json!({
            "appCode": "CoW Swap",
            "metadata": { "quote": { "slippageBips": 50 }, "shared": { "base": true, "value": "base" } },
            "version": "1.14.0"
        }),
        _ => bounded_json(bytes, 0),
    }
}

fn override_for_seed(seed_class: u8, bytes: &mut Unstructured<'_>) -> AppDataParams {
    match seed_class {
        0 => AppDataParams::default(),
        1 => AppDataParams::default().with_metadata(metadata_from_value(json!({
            "quote": { "nested": { "override": true } }
        }))),
        2 => AppDataParams::default().with_metadata(metadata_from_value(json!({
            "partnerFee": partner_fee_value(false),
            "userConsents": [{ "terms": "QmOverrideTermsHash", "acceptedDate": "2025-11-12T08:30:00Z" }]
        }))),
        3 => AppDataParams::default().with_signer(address(0x22)),
        4 => AppDataParams::default().with_metadata(metadata_from_value(json!({
            "partnerFee": partner_fee_value(false)
        }))),
        5 => AppDataParams::default().with_metadata(metadata_from_value(json!({
            "partnerFee": partner_fee_value(true)
        }))),
        6 => AppDataParams::default().with_hooks(hooks(0x55)),
        7 => AppDataParams::default().with_flashloan(flashloan(0x66)),
        8 => AppDataParams::default().with_metadata(metadata_from_value(json!({
            "shared": { "override": true, "value": "override" }
        }))),
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
        "userConsents",
        "shared",
        "value",
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
        _ => value % 9,
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
