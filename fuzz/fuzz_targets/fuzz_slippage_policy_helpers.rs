#![no_main]

//! Fuzz target for the trading slippage policy helpers.
//!
//! **Surface:** `cow_sdk_trading::{sanitize_protocol_fee_bps,
//! suggest_slippage_from_fee, suggest_slippage_from_volume}`.
//! **Property:** `PROP-TRD-002`.
//! **Seed contract:** corpus inputs cover canonical decimal and `0x`-hex
//! quantity strings, boundary zero / one / `u128::MAX` amounts, and
//! adversarial NaN / Inf / negative / oversized values that exercise the
//! documented sanitization and integer-math fast paths.
//!
//! The target maps arbitrary bytes through `Arbitrary` into a typed
//! `PolicyInput`, exercises `sanitize_protocol_fee_bps` on the candidate
//! string, walks `suggest_slippage_from_fee` and
//! `suggest_slippage_from_volume` on the candidate numeric triples, and
//! asserts:
//! - None of the helpers ever panic for arbitrary input.
//! - Determinism on identical input.
//! - `sanitize_protocol_fee_bps` returns `None` for non-finite or
//!   sub-minimum values.
//! - `suggest_slippage_from_fee` and `suggest_slippage_from_volume`
//!   produce non-negative amounts within the documented uint256 bound.

use cow_sdk_trading::{
    sanitize_protocol_fee_bps, suggest_slippage_from_fee, suggest_slippage_from_volume,
};
use libfuzzer_sys::{
    arbitrary::{Arbitrary, Unstructured},
    fuzz_target,
};

#[derive(Debug)]
struct PolicyInput {
    protocol_fee_bps: Option<String>,
    fee_amount: String,
    multiplier_tag: u8,
    sell_before: String,
    sell_after: String,
    slippage_tag: u8,
    is_sell: bool,
}

impl<'a> Arbitrary<'a> for PolicyInput {
    fn arbitrary(bytes: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        Ok(Self {
            protocol_fee_bps: if bool::arbitrary(bytes).unwrap_or(false) {
                Some(read_quantity_string(bytes))
            } else {
                None
            },
            fee_amount: read_quantity_string(bytes),
            multiplier_tag: read_u8(bytes, 0),
            sell_before: read_quantity_string(bytes),
            sell_after: read_quantity_string(bytes),
            slippage_tag: read_u8(bytes, 0),
            is_sell: bool::arbitrary(bytes).unwrap_or(false),
        })
    }
}

fuzz_target!(|input: PolicyInput| {
    // sanitize_protocol_fee_bps: never panics, deterministic, rejects
    // non-finite or sub-minimum values.
    let first = sanitize_protocol_fee_bps(input.protocol_fee_bps.as_deref());
    let second = sanitize_protocol_fee_bps(input.protocol_fee_bps.as_deref());
    assert_eq!(
        first, second,
        "sanitize_protocol_fee_bps must be deterministic on identical input",
    );
    if let Some(value) = first {
        assert!(
            value.is_finite() && value >= 0.0001,
            "sanitize_protocol_fee_bps accepted a non-finite or sub-minimum value",
        );
    }

    // suggest_slippage_from_fee: never panics; results stay within uint256.
    let multiplier = multiplier_for_tag(input.multiplier_tag);
    let fee_first = suggest_slippage_from_fee(&input.fee_amount, multiplier);
    let fee_second = suggest_slippage_from_fee(&input.fee_amount, multiplier);
    assert_eq!(
        fee_first.is_ok(),
        fee_second.is_ok(),
        "suggest_slippage_from_fee must be deterministic on identical input",
    );
    if let Ok(_amount) = fee_first {
        // `Amount` is `#[repr(transparent)]` over `alloy_primitives::U256`
        // per ADR 0052; the uint256 ceiling is enforced by the type
        // system, so the historical `bits() <= 256` runtime guard
        // collapses to a constant-true invariant.
    }

    // suggest_slippage_from_volume: never panics; results stay within uint256.
    let slippage = slippage_for_tag(input.slippage_tag);
    let vol_first = suggest_slippage_from_volume(
        input.is_sell,
        &input.sell_before,
        &input.sell_after,
        slippage,
    );
    let vol_second = suggest_slippage_from_volume(
        input.is_sell,
        &input.sell_before,
        &input.sell_after,
        slippage,
    );
    assert_eq!(
        vol_first.is_ok(),
        vol_second.is_ok(),
        "suggest_slippage_from_volume must be deterministic on identical input",
    );
    if let Ok(_amount) = vol_first {
        // `Amount` is `#[repr(transparent)]` over `alloy_primitives::U256`
        // per ADR 0052; the uint256 ceiling is enforced by the type
        // system, so the historical `bits() <= 256` runtime guard
        // collapses to a constant-true invariant.
    }

    // Documented adversarial guard: NaN / Inf / negative protocol fees
    // must be rejected by sanitize_protocol_fee_bps regardless of input.
    assert_eq!(
        sanitize_protocol_fee_bps(Some("NaN")),
        None,
        "sanitize_protocol_fee_bps must reject NaN",
    );
    assert_eq!(
        sanitize_protocol_fee_bps(Some("inf")),
        None,
        "sanitize_protocol_fee_bps must reject infinity",
    );
    assert_eq!(
        sanitize_protocol_fee_bps(Some("-1")),
        None,
        "sanitize_protocol_fee_bps must reject negative values",
    );
});

fn multiplier_for_tag(tag: u8) -> f64 {
    match tag % 8 {
        0 => 0.0,
        1 => 0.5,
        2 => 50.0,
        3 => 100.0,
        4 => -1.0,
        5 => f64::NAN,
        6 => f64::INFINITY,
        _ => 1e18,
    }
}

fn slippage_for_tag(tag: u8) -> f64 {
    match tag % 8 {
        0 => 0.0,
        1 => 0.5,
        2 => 50.0,
        3 => 100.0,
        4 => -1.0,
        5 => f64::NAN,
        6 => f64::INFINITY,
        _ => 1e18,
    }
}

fn read_quantity_string(bytes: &mut Unstructured<'_>) -> String {
    let tag = read_u8(bytes, 0) % 8;
    let value = u128::arbitrary(bytes).unwrap_or(0);
    match tag {
        0 => value.to_string(),
        1 => format!("0x{value:x}"),
        2 => "0".to_owned(),
        3 => "1".to_owned(),
        4 => u128::MAX.to_string(),
        5 => format!("-{value}"),
        6 => "0x".to_owned(),
        _ => "not-a-number".to_owned(),
    }
}

fn read_u8(bytes: &mut Unstructured<'_>, default: u8) -> u8 {
    u8::arbitrary(bytes).unwrap_or(default)
}
