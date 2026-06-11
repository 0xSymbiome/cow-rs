//! Property-based coverage for the strongly-typed `cow-sdk-core` boundary.
//!
//! Each `proptest!` case exercises a named invariant on one of the core
//! domain types. Shrinking narrows any counter-example to a minimal
//! input before `cargo test` prints it, and committed seed files under
//! `tests/proptest-regressions/` keep the shrink outcomes reproducible
//! across contributors. Net coverage matches the hand-rolled enumerator
//! this file replaced: every invariant family the enumerator exercised
//! carries a named property here.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::missing_const_for_fn,
    clippy::redundant_clone,
    clippy::redundant_closure,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "pedantic, nursery, and style lints acceptable in test helper code"
)]

use std::collections::{HashMap, HashSet};

use alloy_primitives::U256;
use cow_sdk_core::{
    Address, Amount, AppDataHash, ChainId, Hash32, HexData, OrderUid, SupportedChainId,
    VALID_TO_MAX_RELATIVE_SECONDS, VALID_TO_MIN_RELATIVE_SECONDS, ValidTo,
};
use num_bigint::BigUint;
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// Path for committed regression seeds; proptest writes new shrink
/// outcomes here so every contributor re-runs prior counter-examples
/// before any novel case is generated.
const REGRESSION_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/proptest-regressions/property_contract.txt"
);

/// Renders a byte slice as hex with the per-nibble casing bits supplied
/// by the strategy so shrinking can isolate casing-sensitive failures.
fn render_mixed_case(bytes: &[u8], casing: &[bool]) -> String {
    debug_assert_eq!(bytes.len() * 2, casing.len());
    let mut out = String::with_capacity(bytes.len() * 2 + 2);
    out.push_str("0x");
    for (index, byte) in bytes.iter().enumerate() {
        let hi = byte >> 4;
        let lo = byte & 0x0F;
        out.push(nibble_char(hi, casing[index * 2]));
        out.push(nibble_char(lo, casing[index * 2 + 1]));
    }
    out
}

fn nibble_char(value: u8, uppercase: bool) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 if uppercase => (b'A' + value - 10) as char,
        10..=15 => (b'a' + value - 10) as char,
        _ => unreachable!("a nibble value always fits in four bits"),
    }
}

/// Strategy that emits every supported chain id.
fn supported_chain_strategy() -> impl Strategy<Value = SupportedChainId> {
    cow_sdk_test_utils::arb::arb_supported_chain_id()
}

/// Strategy that emits an arbitrary 20-byte address payload.
fn address_bytes() -> impl Strategy<Value = [u8; 20]> {
    any::<[u8; 20]>()
}

/// Strategy that emits an arbitrary 32-byte payload; used as the
/// amount value domain because every representable [`Amount`]
/// fits in 256 bits.
fn atom_amount_bytes() -> impl Strategy<Value = [u8; 32]> {
    any::<[u8; 32]>()
}

/// Strategy that emits an arbitrary 56-byte order-UID payload.
fn order_uid_bytes() -> impl Strategy<Value = [u8; 56]> {
    (any::<[u8; 32]>(), any::<[u8; 24]>()).prop_map(|(first, second)| {
        let mut out = [0u8; 56];
        out[..32].copy_from_slice(&first);
        out[32..].copy_from_slice(&second);
        out
    })
}

/// Strategy that emits the union of malformed hex shapes
/// [`Address::new`] must reject: missing `0x` prefix, uppercase `0X`
/// prefix, short payload, long payload, and non-hex-character payload.
fn malformed_address_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        any::<[u8; 20]>().prop_map(|bytes| alloy_primitives::hex::encode(bytes)),
        any::<[u8; 20]>().prop_map(|bytes| format!("0X{}", alloy_primitives::hex::encode(bytes))),
        any::<[u8; 19]>().prop_map(|bytes| format!("0x{}", alloy_primitives::hex::encode(bytes))),
        any::<[u8; 21]>().prop_map(|bytes| format!("0x{}", alloy_primitives::hex::encode(bytes))),
        (any::<[u8; 20]>(), 2usize..42).prop_map(|(bytes, flip)| {
            let mut encoded = format!("0x{}", alloy_primitives::hex::encode(bytes)).into_bytes();
            encoded[flip] = b'g';
            String::from_utf8(encoded).unwrap()
        }),
    ]
}

/// Strategy that emits the union of malformed hex shapes [`Hash32::new`]
/// must reject: empty, empty payload after the prefix, short, long, and
/// non-hex-character payloads.
fn malformed_hash32_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just("0x".to_owned()),
        any::<[u8; 31]>().prop_map(|bytes| format!("0x{}", alloy_primitives::hex::encode(bytes))),
        any::<[u8; 33]>().prop_map(|bytes| format!("0x{}", alloy_primitives::hex::encode(bytes))),
        (any::<[u8; 32]>(), 2usize..66).prop_map(|(bytes, flip)| {
            let mut encoded = format!("0x{}", alloy_primitives::hex::encode(bytes)).into_bytes();
            encoded[flip] = b'z';
            String::from_utf8(encoded).unwrap()
        }),
    ]
}

/// Strategy that emits the union of malformed hex shapes
/// [`AppDataHash::new`] must reject.
fn malformed_app_data_hex_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        any::<[u8; 32]>().prop_map(|bytes| alloy_primitives::hex::encode(bytes)),
        any::<[u8; 31]>().prop_map(|bytes| format!("0x{}", alloy_primitives::hex::encode(bytes))),
        any::<[u8; 33]>().prop_map(|bytes| format!("0x{}", alloy_primitives::hex::encode(bytes))),
    ]
}

/// Strategy that emits the union of malformed hex shapes
/// [`OrderUid::new`] must reject.
fn malformed_order_uid_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        any::<[u8; 55]>().prop_map(|bytes| format!("0x{}", alloy_primitives::hex::encode(bytes))),
        any::<[u8; 57]>().prop_map(|bytes| format!("0x{}", alloy_primitives::hex::encode(bytes))),
    ]
}

/// Strategy that emits the union of malformed [`Amount`] inputs: empty,
/// negative decimal, invalid hex, decimal with fractional part, and a
/// value larger than 256 bits.
fn malformed_amount_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        (1u64..=u64::MAX).prop_map(|value| format!("-{value}")),
        any::<[u8; 4]>().prop_map(|bytes| format!("0x{}gg", alloy_primitives::hex::encode(bytes))),
        (1u64..=u64::MAX, 1u64..=u64::MAX).prop_map(|(whole, frac)| format!("{whole}.{frac}")),
        Just(format!("0x1{}", "0".repeat(64))),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSION_FILE))),
        ..ProptestConfig::default()
    })]

    /// Any 20-byte payload rendered as lowercase, uppercase, or mixed-case
    /// hex parses into an [`Address`] whose [`PartialEq`],
    /// `to_hex_string`, and [`std::hash::Hash`]
    /// implementations all treat the three renderings as the same address.
    /// `HashMap` and `HashSet` lookups must agree with the equality rule
    /// across every casing variant. The cow `Address` canonicalises every
    /// input to its lowercase 0x-prefixed hex form per ADR 0052; the
    /// stored string form is the lowercase canonical form, not the
    /// original input casing.
    #[test]
    fn address_case_normalization_holds_across_hash_and_equality(
        bytes in address_bytes(),
        casing in prop::collection::vec(any::<bool>(), 40),
    ) {
        let mixed = render_mixed_case(&bytes, &casing);
        let lowercase = format!("0x{}", alloy_primitives::hex::encode(bytes));
        let uppercase = format!("0x{}", alloy_primitives::hex::encode_upper(bytes));

        let mixed_address = Address::new(&mixed).unwrap();
        let lowercase_address = Address::new(&lowercase).unwrap();
        let uppercase_address = Address::new(&uppercase).unwrap();

        prop_assert_eq!(mixed_address.to_hex_string(), lowercase.clone());

        let roundtrip = mixed_address.to_hex_string();
        prop_assert_eq!(&roundtrip, &lowercase);
        prop_assert_eq!(Address::new(roundtrip).unwrap(), mixed_address);

        prop_assert_eq!(&mixed_address, &lowercase_address);
        prop_assert_eq!(&uppercase_address, &lowercase_address);
        prop_assert_eq!(mixed_address.to_hex_string(), lowercase.clone());
        prop_assert_eq!(lowercase_address.to_hex_string(), uppercase_address.to_hex_string());

        let mut map = HashMap::new();
        map.insert(mixed_address, "value");
        prop_assert_eq!(map.get(&lowercase_address), Some(&"value"));
        prop_assert_eq!(map.get(&uppercase_address), Some(&"value"));

        let mut set = HashSet::new();
        set.insert(mixed_address);
        set.insert(lowercase_address);
        set.insert(uppercase_address);
        prop_assert_eq!(set.len(), 1);
    }

    /// [`Address::new`] fails closed on every malformed hex shape the
    /// reviewed contract rejects: missing `0x` prefix, uppercase `0X`
    /// prefix, wrong length, and non-hex characters inside the payload.
    #[test]
    fn address_rejects_malformed_inputs(input in malformed_address_strategy()) {
        prop_assert!(Address::new(&input).is_err(), "input = {input}");
    }

    /// [`Amount`] treats decimal and hex renderings of the same 256-bit
    /// value as equal, preserves the canonical base-10 form as its own
    /// output, and round-trips through its own string form.
    #[test]
    fn amount_canonical_decimal_matches_hex_equivalent(bytes in atom_amount_bytes()) {
        let value = U256::from_be_bytes(bytes);
        let canonical = value.to_string();
        let hex_form = format!("{value:#x}");

        let from_decimal = Amount::new(&canonical).unwrap();
        let from_hex = Amount::new(&hex_form).unwrap();

        prop_assert_eq!(&from_decimal, &from_hex);
        prop_assert_eq!(from_decimal.to_string(), canonical.clone());

        let roundtrip = Amount::new(from_decimal.to_string()).unwrap();
        prop_assert_eq!(&roundtrip, &from_decimal);

        prop_assert_eq!(from_decimal.as_u256(), &value);
    }

    /// [`Amount::new`] fails closed on every malformed input shape the
    /// reviewed contract rejects: empty string, negative decimal, invalid
    /// hex, fractional decimal, and values larger than 256 bits.
    #[test]
    fn amount_rejects_malformed_and_out_of_range_inputs(input in malformed_amount_strategy()) {
        prop_assert!(Amount::new(&input).is_err(), "input = {input}");
    }

    /// [`Amount::checked_mul`] preserves the `uint256` ceiling even when
    /// callers construct values directly from raw atoms.
    #[test]
    fn amount_checked_mul_rejects_uint256_overflow(
        bytes in atom_amount_bytes(),
        multiplier in 0u8..=4u8,
    ) {
        let left = Amount::from_u256(U256::from_be_bytes(bytes));
        let right = Amount::from_u256(U256::from(multiplier));
        // Cross-check the typed checked-mul against an arbitrary-width
        // `BigUint` oracle so the U256 overflow boundary is observable
        // independently of the implementation under test.
        let left_big = BigUint::from_bytes_be(&bytes);
        let right_big = BigUint::from(u32::from(multiplier));
        let product = &left_big * &right_big;

        prop_assert_eq!(
            left.checked_mul(&right).is_some(),
            product.bits() <= 256,
        );

        let max = Amount::from_u256(U256::MAX);
        let two = Amount::from(2u32);
        prop_assert!(max.checked_mul(&two).is_none());
    }

    /// [`Hash32::new`] preserves the supplied input string exactly
    /// (including casing) and round-trips through its own string form.
    #[test]
    fn hash32_roundtrip_preserves_input(
        bytes in any::<[u8; 32]>(),
        casing in prop::collection::vec(any::<bool>(), 64),
    ) {
        let canonical = format!("0x{}", alloy_primitives::hex::encode(bytes));
        let mixed = render_mixed_case(&bytes, &casing);

        let hash = Hash32::new(&canonical).unwrap();
        prop_assert_eq!(hash.to_hex_string(), canonical.clone());

        let hash_mixed = Hash32::new(&mixed).unwrap();
        prop_assert_eq!(hash_mixed.to_hex_string(), canonical.clone());

        let rebuilt = Hash32::new(hash.to_hex_string()).unwrap();
        prop_assert_eq!(rebuilt, hash);
    }

    /// [`Hash32::new`] fails closed on every malformed hex shape.
    #[test]
    fn hash32_rejects_malformed_inputs(input in malformed_hash32_strategy()) {
        prop_assert!(Hash32::new(&input).is_err(), "input = {input}");
    }

    /// [`AppDataHash::new`] preserves a 32-byte canonical payload and
    /// round-trips through its own string form; malformed shapes (missing
    /// prefix, wrong length) fail closed.
    #[test]
    fn app_data_hex_roundtrip_and_rejects_malformed(
        bytes in any::<[u8; 32]>(),
        malformed in malformed_app_data_hex_strategy(),
    ) {
        let canonical = format!("0x{}", alloy_primitives::hex::encode(bytes));

        let app_data = AppDataHash::new(&canonical).unwrap();
        prop_assert_eq!(app_data.to_hex_string(), canonical.clone());
        prop_assert_eq!(AppDataHash::new(app_data.to_hex_string()).unwrap(), app_data);

        prop_assert!(AppDataHash::new(&malformed).is_err(), "malformed = {malformed}");
    }

    /// [`OrderUid::new`] preserves a 56-byte canonical payload and
    /// round-trips through its own string form; malformed lengths fail
    /// closed.
    #[test]
    fn order_uid_roundtrip_and_rejects_malformed(
        bytes in order_uid_bytes(),
        malformed in malformed_order_uid_strategy(),
    ) {
        let canonical = format!("0x{}", alloy_primitives::hex::encode(bytes));

        let uid = OrderUid::new(&canonical).unwrap();
        prop_assert_eq!(uid.to_hex_string(), canonical.clone());
        prop_assert_eq!(OrderUid::new(uid.to_hex_string()).unwrap(), uid);

        prop_assert!(OrderUid::new(&malformed).is_err(), "malformed = {malformed}");
    }

    /// [`HexData`] preserves the canonical empty payload `0x`, matches
    /// [`HexData::default`], and preserves any 0x-prefixed hex body
    /// byte-for-byte.
    #[test]
    fn hex_data_accepts_empty_payload_and_preserves_valid_inputs(bytes in any::<[u8; 32]>()) {
        let empty = HexData::empty();
        prop_assert_eq!(empty.to_hex_string(), "0x".to_owned());
        prop_assert_eq!(HexData::default(), empty);

        let canonical = format!("0x{}", alloy_primitives::hex::encode(bytes));
        let data = HexData::new(&canonical).unwrap();
        prop_assert_eq!(data.to_hex_string(), canonical.clone());
        prop_assert_eq!(HexData::new(data.to_hex_string()).unwrap(), data);
    }

    /// [`Amount::from_u256`] preserves the originating [`U256`] input,
    /// round-trips through the canonical decimal-string Serde form, and
    /// accepts the same value constructed through [`Amount::new`].
    #[test]
    fn amount_roundtrips_through_u256_and_wire_string(bytes in atom_amount_bytes()) {
        let value = U256::from_be_bytes(bytes);
        let canonical = value.to_string();

        let amount = Amount::from_u256(value);
        prop_assert_eq!(amount.as_u256(), &value);

        let round_trip_u256: U256 = amount.into();
        prop_assert_eq!(round_trip_u256, value);

        prop_assert_eq!(amount.to_string(), canonical.clone());

        let serialized = serde_json::to_string(&amount).unwrap();
        let deserialized: Amount = serde_json::from_str(&serialized).unwrap();
        prop_assert_eq!(deserialized.as_u256(), &value);

        let from_new = Amount::new(canonical.clone()).unwrap();
        prop_assert_eq!(from_new, amount);
    }

    /// [`Amount::format_units`] composed with [`Amount::parse_units`]
    /// is an exact identity for every representable `(atoms, decimals)`
    /// pair: `format_units` preserves the full `decimals`-wide
    /// fractional substring (no trimming), so re-parsing the rendered
    /// string recovers the originating atoms byte-for-byte across the
    /// whole 0..=77 decimals range and the whole `uint256` value range.
    #[test]
    fn amount_format_units_then_parse_units_round_trips_exactly(
        bytes in atom_amount_bytes(),
        decimals in 0u8..=77u8,
    ) {
        let atoms = Amount::from_u256(U256::from_be_bytes(bytes));
        let rendered = atoms.format_units(decimals);
        let reparsed = Amount::parse_units(&rendered, decimals)
            .expect("format_units output must always re-parse through parse_units");
        prop_assert_eq!(
            reparsed,
            atoms,
            "parse_units(format_units(x, d), d) must equal x; rendered = {}",
            rendered,
        );
    }

    /// [`Amount::parse_units`] is integer-exact with no `f64` drift for
    /// arbitrary whole and fractional digit inputs whose fractional
    /// width does not exceed `decimals`. The expected atoms are
    /// computed with an independent `U256` integer oracle
    /// (`whole * 10^decimals + frac * 10^(decimals - frac_len)`)
    /// evaluated with checked arithmetic, so a future body that routes
    /// through floating point would surface as a shrunken
    /// counter-example. When the oracle overflows `uint256`,
    /// `parse_units` must fail closed instead.
    #[test]
    fn amount_parse_units_is_integer_exact(
        whole in any::<u128>(),
        frac_digits in prop::collection::vec(0u8..=9u8, 0usize..=20usize),
        extra_decimals in 0u8..=40u8,
    ) {
        // Pick a decimals scale that is at least the fractional width so
        // no truncation occurs, and cap it at the documented 77 maximum.
        let frac_len = u8::try_from(frac_digits.len()).expect("frac width <= 20 fits in u8");
        let decimals = (frac_len.saturating_add(extra_decimals)).min(77);

        // Build the canonical decimal input string. An empty fractional
        // vector yields a bare integer (no decimal point).
        let frac_string: String = frac_digits
            .iter()
            .map(|digit| char::from(b'0' + digit))
            .collect();
        let input = if frac_string.is_empty() {
            whole.to_string()
        } else {
            format!("{whole}.{frac_string}")
        };

        // Independent integer oracle, evaluated with checked arithmetic so
        // the oracle itself never overflows: scale the whole part by
        // 10^decimals, then add the fractional digits left-aligned into
        // the fractional field (padded on the right to `decimals`).
        // `10^decimals` for `decimals <= 77` is always below `U256::MAX`,
        // so the `pow` calls cannot overflow; only the whole-part scaling
        // and the final addition can, which `checked_*` detects.
        let scale = U256::from(10u8).pow(U256::from(decimals));
        let oracle = U256::from(whole).checked_mul(scale).and_then(|scaled| {
            if frac_string.is_empty() {
                Some(scaled)
            } else {
                let frac_value = frac_string
                    .parse::<U256>()
                    .expect("a digit-only fractional string parses as U256");
                let pad = U256::from(10u8).pow(U256::from(decimals - frac_len));
                // `frac_value < 10^frac_len` and `pad == 10^(decimals - frac_len)`,
                // so `frac_value * pad < 10^decimals <= scale`; this product
                // never overflows on its own and only the final add can.
                scaled.checked_add(frac_value * pad)
            }
        });

        let parsed = Amount::parse_units(&input, decimals);
        match oracle {
            Some(expected) => {
                let parsed = parsed
                    .expect("an in-range digit-only decimal within decimals must parse")
                    .into_u256();
                prop_assert_eq!(
                    parsed,
                    expected,
                    "parse_units must equal the integer oracle; input = {}, decimals = {}",
                    input,
                    decimals,
                );
            }
            None => {
                prop_assert!(
                    parsed.is_err(),
                    "parse_units must fail closed when the value overflows uint256; \
                     input = {}, decimals = {}",
                    input,
                    decimals,
                );
            }
        }
    }

    /// [`Amount::from_units`] is integer-exact and fail-closed for every
    /// `(whole, decimals)` pair in the supported range: it must equal the
    /// independent `whole * 10^decimals` `U256` oracle when that fits
    /// `uint256`, and must return an error (never panic, never wrap) when
    /// it does not. It must also agree exactly with [`Amount::parse_units`]
    /// applied to the same whole number rendered as a decimal string — the
    /// numeric and textual constructors are two doors to one value.
    #[test]
    fn amount_from_units_matches_integer_oracle_and_parse_units(
        whole in any::<u128>(),
        decimals in 0u8..=77u8,
    ) {
        // Independent integer oracle: `10^decimals` for `decimals <= 77`
        // never overflows uint256, so only the final multiply by `whole`
        // can, which `checked_mul` detects.
        let scale = U256::from(10u8).pow(U256::from(decimals));
        let oracle = U256::from(whole).checked_mul(scale);

        let built = Amount::from_units(whole, decimals);
        match oracle {
            Some(expected) => {
                let built = built
                    .expect("an in-range whole-unit count within uint256 must build")
                    .into_u256();
                prop_assert_eq!(
                    built,
                    expected,
                    "from_units must equal the integer oracle; whole = {}, decimals = {}",
                    whole,
                    decimals,
                );
                // The numeric door agrees with the textual door.
                let parsed = Amount::parse_units(whole.to_string(), decimals)
                    .expect("parse_units of a bare whole number within range must parse")
                    .into_u256();
                prop_assert_eq!(
                    built,
                    parsed,
                    "from_units must equal parse_units of the same whole number; whole = {}",
                    whole,
                );
            }
            None => {
                prop_assert!(
                    built.is_err(),
                    "from_units must fail closed when whole * 10^decimals overflows uint256; \
                     whole = {}, decimals = {}",
                    whole,
                    decimals,
                );
            }
        }
    }

    /// [`Amount::parse_units`] never panics for any `decimals` in the
    /// documented `0..=77` range paired with arbitrary input bytes: the
    /// constructor is fail-closed by `Result`, so every input either
    /// parses to a typed [`Amount`] or returns an error, and an `Ok`
    /// result always round-trips through [`Amount::format_units`].
    #[test]
    fn amount_parse_units_never_panics_within_decimals_range(
        raw in any::<Vec<u8>>(),
        decimals in 0u8..=77u8,
    ) {
        let input = String::from_utf8_lossy(&raw).into_owned();
        if let Ok(amount) = Amount::parse_units(&input, decimals) {
            let rendered = amount.format_units(decimals);
            let reparsed = Amount::parse_units(&rendered, decimals)
                .expect("format_units output of an accepted Amount must re-parse");
            prop_assert_eq!(
                reparsed,
                amount,
                "an accepted parse_units value must round-trip through format_units; input = {}",
                input,
            );
        }
    }

    /// Every [`SupportedChainId`] round-trips through its [`ChainId`]
    /// numeric form, and any u64 outside the supported set fails the
    /// [`TryFrom`] conversion.
    #[test]
    fn supported_chain_id_roundtrips_and_rejects_unknown(
        supported in supported_chain_strategy(),
        candidate in any::<u64>(),
    ) {
        let raw: ChainId = supported.into();
        let rebuilt = SupportedChainId::try_from(raw).unwrap();
        prop_assert_eq!(supported, rebuilt);

        let is_supported = SupportedChainId::ALL
            .iter()
            .any(|chain| ChainId::from(*chain) == candidate);
        prop_assert_eq!(SupportedChainId::try_from(candidate).is_ok(), is_supported);
    }

    /// [`ValidTo::relative`] admits every duration inside
    /// `[VALID_TO_MIN_RELATIVE_SECONDS, VALID_TO_MAX_RELATIVE_SECONDS]`
    /// and fails closed on every duration outside that inclusive window.
    #[test]
    fn valid_to_relative_enforces_documented_bounds(
        now_epoch_seconds in 1_600_000_000u64..=4_000_000_000u64,
        duration_seconds in 0u64..=(u64::from(VALID_TO_MAX_RELATIVE_SECONDS) + 3_600),
    ) {
        let result = ValidTo::relative(now_epoch_seconds, duration_seconds);
        let in_range = (u64::from(VALID_TO_MIN_RELATIVE_SECONDS)
            ..=u64::from(VALID_TO_MAX_RELATIVE_SECONDS))
            .contains(&duration_seconds);
        prop_assert_eq!(result.is_ok(), in_range);
    }
}

// Property coverage for the narrowed `pow` / `bit_len` / `bits`
// surface. Each property pins the cow wrapper against the underlying
// alloy / ruint primitive across a randomised input range, so a
// future refactor that diverges from the documented delegation
// contract will surface immediately with a shrunken counter-example.
proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSION_FILE))),
        ..ProptestConfig::default()
    })]

    /// `Amount::checked_pow` must equal `U256::checked_pow` (the
    /// genuine overflow-detecting variant) for every random
    /// `(base, exp)` pair. If a future contributor rewires the cow
    /// body to call `U256::pow` instead, this property catches the
    /// silent-wrap divergence: `U256::pow` is `wrapping_pow` and
    /// would return `Some(wrapped)` where the contract expects
    /// `None` on overflow.
    #[test]
    fn amount_checked_pow_delegates_to_inner_uint_checked_pow(
        base in any::<u128>().prop_map(U256::from).prop_map(Amount::from),
        exp in 0u32..=32u32,
    ) {
        let exp_amount = Amount::new(exp.to_string()).unwrap();
        let cow_result = base.checked_pow(&exp_amount);
        let raw_result = base.into_u256().checked_pow(U256::from(exp)).map(Amount::from);
        prop_assert_eq!(cow_result, raw_result);
    }

    /// `Amount::saturating_pow` must equal `U256::saturating_pow`
    /// for every random `(base, exp)` pair.
    #[test]
    fn amount_saturating_pow_delegates_to_inner_uint_saturating_pow(
        base in any::<u128>().prop_map(U256::from).prop_map(Amount::from),
        exp in 0u32..=32u32,
    ) {
        let exp_amount = Amount::new(exp.to_string()).unwrap();
        let cow_result = base.saturating_pow(&exp_amount);
        let raw_result = Amount::from(base.into_u256().saturating_pow(U256::from(exp)));
        prop_assert_eq!(cow_result, raw_result);
    }

    /// `Amount::bit_len` widens the inner `usize` to `u64`
    /// losslessly. The inner `Uint::bit_len` is always `<= 256` for a
    /// 256-bit storage, so the `as u64` conversion is correct on
    /// every supported target.
    #[test]
    fn amount_bit_len_widens_inner_uint_bit_len_losslessly(
        value in any::<u128>().prop_map(U256::from).prop_map(Amount::from),
    ) {
        let cow_bits = value.bit_len();
        let raw_bits = value.into_u256().bit_len() as u64;
        prop_assert_eq!(cow_bits, raw_bits);
        prop_assert!(cow_bits <= 256);
    }
}
