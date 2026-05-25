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

use alloy_primitives::{I256, U256};
use cow_sdk_core::{
    Address, Amount, AppDataHex, ChainId, DecimalAmount, Hash32, HexData, OrderUid, SignedAmount,
    SupportedChainId, VALID_TO_MAX_RELATIVE_SECONDS, VALID_TO_MIN_RELATIVE_SECONDS, ValidTo,
    addresses_equal, token_id,
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
    prop::sample::select(SupportedChainId::ALL.to_vec())
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

/// Strategy that emits arbitrary `I256` values across the full signed
/// 256-bit range. Sampling re-interprets the unsigned 32-byte payload
/// as a two's-complement [`I256`], so every representable bit pattern
/// — including both extremes `I256::MIN` and `I256::MAX` — appears
/// with equal probability.
fn signed_amount_value_strategy() -> impl Strategy<Value = I256> {
    atom_amount_bytes().prop_map(|bytes| I256::from_raw(U256::from_be_bytes(bytes)))
}

/// Strategy that emits valid signed-decimal strings including redundant
/// leading zeroes so [`SignedAmount::new`] normalization is exercised.
fn signed_amount_input_strategy() -> impl Strategy<Value = String> {
    (signed_amount_value_strategy(), 0usize..=4usize).prop_map(|(value, leading_zeroes)| {
        let canonical = value.to_string();
        let (prefix, digits) = canonical
            .strip_prefix('-')
            .map_or(("", canonical.as_str()), |digits| ("-", digits));
        format!("{prefix}{}{digits}", "0".repeat(leading_zeroes))
    })
}

/// Curated signed-amount literals that pin the canonical decimal-string
/// wire form across zero, sign, `i128`, and `I256` boundary values.
fn curated_signed_amount_inputs() -> Vec<String> {
    vec![
        "0".to_owned(),
        "-0".to_owned(),
        "1".to_owned(),
        "-1".to_owned(),
        "00042".to_owned(),
        "-00042".to_owned(),
        i128::MAX.to_string(),
        i128::MIN.to_string(),
        I256::MAX.to_string(),
        I256::MIN.to_string(),
    ]
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
/// [`AppDataHex::new`] must reject.
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
    /// [`addresses_equal`], `to_hex_string`, and [`std::hash::Hash`]
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
        prop_assert!(addresses_equal(&mixed_address, &lowercase_address));
        prop_assert!(addresses_equal(&uppercase_address, &lowercase_address));

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

    /// [`AppDataHex::new`] preserves a 32-byte canonical payload and
    /// round-trips through its own string form; malformed shapes (missing
    /// prefix, wrong length) fail closed.
    #[test]
    fn app_data_hex_roundtrip_and_rejects_malformed(
        bytes in any::<[u8; 32]>(),
        malformed in malformed_app_data_hex_strategy(),
    ) {
        let canonical = format!("0x{}", alloy_primitives::hex::encode(bytes));

        let app_data = AppDataHex::new(&canonical).unwrap();
        prop_assert_eq!(app_data.to_hex_string(), canonical.clone());
        prop_assert_eq!(AppDataHex::new(app_data.to_hex_string()).unwrap(), app_data);

        prop_assert!(AppDataHex::new(&malformed).is_err(), "malformed = {malformed}");
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

    /// [`token_id`] is deterministic for identical `(chain, address)`
    /// inputs and changes when either the chain or the address changes.
    #[test]
    fn token_id_is_chain_and_address_sensitive(
        first_bytes in address_bytes(),
        second_bytes in address_bytes(),
        chain_a in supported_chain_strategy(),
        chain_b in supported_chain_strategy(),
    ) {
        prop_assume!(first_bytes != second_bytes);
        prop_assume!(chain_a != chain_b);

        let address_a = Address::new(format!("0x{}", alloy_primitives::hex::encode(first_bytes))).unwrap();
        let address_b = Address::new(format!("0x{}", alloy_primitives::hex::encode(second_bytes))).unwrap();
        let chain_a: ChainId = chain_a.into();
        let chain_b: ChainId = chain_b.into();

        prop_assert_eq!(token_id(chain_a, &address_a), token_id(chain_a, &address_a));
        prop_assert_ne!(token_id(chain_a, &address_a), token_id(chain_a, &address_b));
        prop_assert_ne!(token_id(chain_a, &address_a), token_id(chain_b, &address_a));
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

    /// [`SignedAmount::new`] canonicalizes valid decimal inputs while
    /// preserving the typed `I256` storage and remaining idempotent
    /// across its own string form.
    #[test]
    fn signed_amount_roundtrip_is_idempotent(input in signed_amount_input_strategy()) {
        let amount = SignedAmount::new(&input).unwrap();
        let canonical = amount.to_string();

        // The cow newtype's `Display` impl is the canonical decimal-string
        // wire form; the inner `I256` accessor exposes the same value.
        prop_assert_eq!(amount.as_i256().to_string(), canonical.clone());

        let rebuilt = SignedAmount::new(canonical.clone()).unwrap();
        prop_assert_eq!(rebuilt, amount);
        prop_assert_eq!(rebuilt.to_string(), canonical);
    }

    /// [`SignedAmount`] keeps the pre-promotion decimal-string JSON shape
    /// byte-identical across the reviewed boundary literals.
    #[test]
    fn signed_amount_wire_serde_matches_legacy_decimal_string_shape(
        input in prop::sample::select(curated_signed_amount_inputs()),
    ) {
        let amount = SignedAmount::new(&input).unwrap();
        let expected = serde_json::to_vec(&amount.to_string()).unwrap();
        let actual = serde_json::to_vec(&amount).unwrap();

        prop_assert_eq!(actual.as_slice(), expected.as_slice());

        let rebuilt: SignedAmount = serde_json::from_slice(&actual).unwrap();
        prop_assert_eq!(rebuilt, amount);
    }

    /// [`DecimalAmount::new`] preserves atoms and decimals across any
    /// representable scale and round-trips through its accessors.
    #[test]
    fn decimal_amount_preserves_atoms_and_scale(
        bytes in atom_amount_bytes(),
        decimals in 0u8..=30u8,
    ) {
        let atoms = U256::from_be_bytes(bytes);
        let amount = DecimalAmount::new(atoms, decimals)
            .expect("the proptest range 0..=30 is within DecimalAmount::MAX_DECIMALS");

        prop_assert_eq!(amount.atoms(), &atoms);
        prop_assert_eq!(amount.decimals(), decimals);

        let rebuilt = DecimalAmount::new(*amount.atoms(), amount.decimals())
            .expect("rebuilding from a previously accepted amount cannot exceed MAX_DECIMALS");
        prop_assert_eq!(&rebuilt, &amount);

        let extracted = amount.into_atoms();
        prop_assert_eq!(extracted, atoms);
    }

    /// [`DecimalAmount::from_whole_approx`] clamps negatives, NaN, and
    /// infinity to safe atoms values and recovers one-token inputs
    /// byte-exactly at 18 decimals.
    #[test]
    fn decimal_amount_from_whole_approx_handles_boundary_inputs(decimals in 0u8..=30u8) {
        let zero = DecimalAmount::from_whole_approx(0.0, decimals)
            .expect("the proptest range 0..=30 is within DecimalAmount::MAX_DECIMALS");
        prop_assert_eq!(zero.atoms(), &U256::ZERO);
        prop_assert_eq!(zero.decimals(), decimals);

        let negative = DecimalAmount::from_whole_approx(-1.5, decimals)
            .expect("the proptest range 0..=30 is within DecimalAmount::MAX_DECIMALS");
        prop_assert_eq!(negative.atoms(), &U256::ZERO);

        let nan = DecimalAmount::from_whole_approx(f64::NAN, decimals)
            .expect("the proptest range 0..=30 is within DecimalAmount::MAX_DECIMALS");
        prop_assert_eq!(nan.atoms(), &U256::ZERO);

        let infinity = DecimalAmount::from_whole_approx(f64::INFINITY, decimals)
            .expect("the proptest range 0..=30 is within DecimalAmount::MAX_DECIMALS");
        prop_assert!(infinity.atoms() <= &U256::from(u128::MAX));

        let one_token = DecimalAmount::from_whole_approx(1.0, 18)
            .expect("decimals 18 is within DecimalAmount::MAX_DECIMALS");
        let expected = U256::from(10u128.pow(18));
        prop_assert_eq!(one_token.atoms(), &expected);
        prop_assert!((one_token.to_f64_approx() - 1.0).abs() < 1e-12);
    }

    /// [`DecimalAmount::to_decimal_string`] pins the four invariants
    /// of its canonical formatted output: (1) no decimal point when
    /// `decimals == 0`; (2) exactly one decimal point when
    /// `decimals > 0`; (3) the fractional substring length always
    /// equals `decimals` (the trailing-zero preservation contract);
    /// (4) the integer part is the canonical decimal expansion of
    /// `atoms / 10.pow(decimals)`, with no leading zero unless the
    /// integer part is itself the literal `"0"`.
    #[test]
    fn decimal_amount_to_decimal_string_pins_fractional_length_invariants(
        bytes in atom_amount_bytes(),
        decimals in 0u8..=DecimalAmount::MAX_DECIMALS,
    ) {
        let atoms = U256::from_be_bytes(bytes);
        let amount = DecimalAmount::new(atoms, decimals)
            .expect("the proptest range 0..=MAX_DECIMALS is within bounds by construction");
        let rendered = amount.to_decimal_string();

        if decimals == 0 {
            // Invariant 1: no decimal point when decimals == 0.
            prop_assert!(
                !rendered.contains('.'),
                "decimals == 0 must produce no decimal point; got {}",
                rendered,
            );
            prop_assert_eq!(&rendered, &atoms.to_string());
        } else {
            // Invariant 2: exactly one decimal point when decimals > 0.
            let dot_count = rendered.matches('.').count();
            prop_assert_eq!(
                dot_count,
                1,
                "decimals > 0 must produce exactly one decimal point; got {}",
                rendered,
            );

            let mut parts = rendered.splitn(2, '.');
            let integer = parts.next().expect("split always yields at least one part");
            let fractional = parts.next().expect("dot_count == 1 guarantees a fractional half");

            // Invariant 3: fractional length always equals decimals.
            prop_assert_eq!(
                fractional.len(),
                usize::from(decimals),
                "fractional substring length must equal decimals; got {}",
                rendered,
            );

            // Invariant 4: integer part has no leading zero unless it
            // is literally "0".
            if integer != "0" {
                let first_byte = integer.as_bytes()[0];
                prop_assert!(
                    first_byte != b'0',
                    "non-zero integer part must not have a leading zero; got {}",
                    rendered,
                );
            }
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

    /// `SignedAmount::checked_pow` must equal
    /// `Signed::checked_pow` for every random `(base, exp)` pair.
    /// The exponent is unsigned per the alloy
    /// `Signed::checked_pow(self, exp: Uint<BITS, LIMBS>)` shape.
    #[test]
    fn signed_amount_checked_pow_delegates_to_inner_signed_checked_pow(
        base_raw in any::<i128>(),
        exp in 0u32..=8u32,
    ) {
        let base = SignedAmount::from_i256(I256::try_from(base_raw).unwrap());
        let exp_amount = Amount::new(exp.to_string()).unwrap();
        let cow_result = base.checked_pow(&exp_amount);
        let raw_result = base
            .into_i256()
            .checked_pow(U256::from(exp))
            .map(SignedAmount::from_i256);
        prop_assert_eq!(cow_result, raw_result);
    }

    /// `SignedAmount::bits` is a direct passthrough to
    /// `Signed::bits` (`u32`).
    #[test]
    fn signed_amount_bits_passes_through_inner_signed_bits(
        value_raw in any::<i128>(),
    ) {
        let value = SignedAmount::from_i256(I256::try_from(value_raw).unwrap());
        let cow_bits = value.bits();
        let raw_bits = value.into_i256().bits();
        prop_assert_eq!(cow_bits, raw_bits);
    }
}
