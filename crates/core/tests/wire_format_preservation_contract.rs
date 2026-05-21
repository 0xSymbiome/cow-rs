//! Wire-form preservation contract for the cow identity primitives.
//!
//! Each test pins the canonical lowercase `0x`-prefixed hexadecimal
//! string that the protocol's TypeScript SDK and the
//! `parity/fixtures/**/*.json` corpora exchange on the wire. The four
//! byte-typed cow newtypes (`Address`, `Hash32`, `HexData`, `OrderUid`)
//! plus `AppDataHash`, `Amount`, and `SignedAmount` ship as
//! `#[repr(transparent)]` wrappers around their `alloy_primitives`
//! counterparts per ADR 0052. Every test exercises both the direct
//! accessor surface and the JSON serde round-trip so the wire bytes
//! stay locked through either path.

use alloy_primitives::I256;

use cow_sdk_core::{Address, Amount, AppDataHash, Hash32, HexData, OrderUid, SignedAmount};

const ADDRESS_HEX: &str = "0x6810e776880c02933d47db1b9fc05908e5386b96";
const ADDRESS_ZERO_HEX: &str = "0x0000000000000000000000000000000000000000";
const HASH32_HEX: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH32_ZERO_HEX: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
const APP_DATA_HEX: &str = "0xd5a25ba2e97094ad7d83dc28a6572da797d6b3e7fc6663bd93efb789fc17e489";
const ORDER_UID_HEX: &str = "0x59920c85de0162e9e55df8d396e75f3b6b7c2dfdb535f03e5c807731c31585eaff714b8b0e2700303ec912bd40496c3997ceea2b616d6710";
const HEX_DATA_FOUR_BYTES: &str = "0xdeadbeef";
const HEX_DATA_EMPTY: &str = "0x";

// ---- Address ----------------------------------------------------------

#[test]
fn address_wire_form_is_lowercase_0x_prefixed_42_chars() {
    let address = Address::new(ADDRESS_HEX).expect("canonical address must parse");
    assert_eq!(address.to_hex_string(), ADDRESS_HEX);

    let json = serde_json::to_string(&address).expect("Address must serialize");
    assert_eq!(json, format!("\"{ADDRESS_HEX}\""));

    let round_trip: Address =
        serde_json::from_str(&json).expect("Address must deserialize round trip");
    assert_eq!(round_trip.to_hex_string(), ADDRESS_HEX);
}

#[test]
fn address_zero_round_trips_with_canonical_zero_bytes() {
    let zero = Address::new(ADDRESS_ZERO_HEX).expect("zero address must parse");
    assert_eq!(zero.to_hex_string(), ADDRESS_ZERO_HEX);
    assert_eq!(zero.byte_length(), 20);

    let json = serde_json::to_string(&zero).expect("zero Address must serialize");
    assert_eq!(json, format!("\"{ADDRESS_ZERO_HEX}\""));
}

#[test]
fn address_rejects_malformed_inputs() {
    assert!(Address::new("").is_err());
    assert!(Address::new("not-hex").is_err());
    assert!(Address::new("0x1234").is_err());
    assert!(Address::new(ADDRESS_HEX.trim_start_matches("0x")).is_err());
    assert!(serde_json::from_str::<Address>(r#""""#).is_err());
    assert!(serde_json::from_str::<Address>(r#""0x1234""#).is_err());
}

// ---- Hash32 -----------------------------------------------------------

#[test]
fn hash32_wire_form_is_lowercase_0x_prefixed_66_chars() {
    let hash = Hash32::new(HASH32_HEX).expect("canonical Hash32 must parse");
    assert_eq!(hash.to_hex_string(), HASH32_HEX);

    let json = serde_json::to_string(&hash).expect("Hash32 must serialize");
    assert_eq!(json, format!("\"{HASH32_HEX}\""));

    let round_trip: Hash32 =
        serde_json::from_str(&json).expect("Hash32 must deserialize round trip");
    assert_eq!(round_trip.to_hex_string(), HASH32_HEX);
}

#[test]
fn hash32_zero_round_trips_with_canonical_zero_bytes() {
    let zero = Hash32::new(HASH32_ZERO_HEX).expect("zero Hash32 must parse");
    assert_eq!(zero.to_hex_string(), HASH32_ZERO_HEX);
    assert_eq!(zero.byte_length(), 32);

    let json = serde_json::to_string(&zero).expect("zero Hash32 must serialize");
    assert_eq!(json, format!("\"{HASH32_ZERO_HEX}\""));
}

#[test]
fn hash32_rejects_malformed_inputs() {
    assert!(Hash32::new("").is_err());
    assert!(Hash32::new("not-hex").is_err());
    assert!(Hash32::new("0x1234").is_err());
    assert!(serde_json::from_str::<Hash32>(r#""0xZZ""#).is_err());
}

// ---- AppDataHash ------------------------------------------------------

#[test]
fn app_data_hash_wire_form_matches_canonical_protocol_constant() {
    let hash = AppDataHash::new(APP_DATA_HEX).expect("canonical AppDataHash must parse");
    assert_eq!(hash.to_hex_string(), APP_DATA_HEX);

    let json = serde_json::to_string(&hash).expect("AppDataHash must serialize");
    assert_eq!(json, format!("\"{APP_DATA_HEX}\""));

    let round_trip: AppDataHash =
        serde_json::from_str(&json).expect("AppDataHash must deserialize round trip");
    assert_eq!(round_trip.to_hex_string(), APP_DATA_HEX);
}

/// Stage B contract row: any uppercase or mixed-case input passes
/// through the strict-newtype constructor and serialises to the
/// canonical lowercase wire form. The cow `AppDataHash` is
/// `#[repr(transparent)]` over [`alloy_primitives::B256`] and forwards
/// `Serialize` / `Deserialize` to the inner alloy primitive whose
/// default already emits the lowercase shape per ADR 0052.
#[test]
fn app_data_hash_mixed_case_input_produces_lowercase_canonical_output() {
    let upper = APP_DATA_HEX.to_ascii_uppercase().replace("0X", "0x");
    let hash = AppDataHash::new(&upper).expect("uppercase AppDataHash must parse");
    assert_eq!(hash.to_hex_string(), APP_DATA_HEX);

    let json = serde_json::to_string(&hash).expect("AppDataHash must serialize");
    assert_eq!(json, format!("\"{APP_DATA_HEX}\""));

    let round_trip: AppDataHash =
        serde_json::from_str(&json).expect("AppDataHash must deserialize round trip");
    assert_eq!(round_trip.to_hex_string(), APP_DATA_HEX);
    assert_eq!(round_trip, hash);
}

// ---- Amount (strict-decimal wire form) --------------------------------

const AMOUNT_DECIMAL: &str = "1234567890";

#[test]
fn amount_wire_form_is_canonical_decimal_string() {
    let amount = Amount::new(AMOUNT_DECIMAL).expect("canonical Amount must parse");
    assert_eq!(amount.to_string(), AMOUNT_DECIMAL);
    assert_eq!(amount.to_decimal_string(), AMOUNT_DECIMAL);

    let json = serde_json::to_string(&amount).expect("Amount must serialize");
    assert_eq!(json, format!("\"{AMOUNT_DECIMAL}\""));

    let round_trip: Amount =
        serde_json::from_str(&json).expect("Amount must deserialize round trip");
    assert_eq!(round_trip.to_string(), AMOUNT_DECIMAL);
    assert_eq!(round_trip, amount);
}

/// Stage B contract row: the cow `Amount` `Deserialize` impl is
/// strict-decimal-fail-closed — JSON payloads carrying `0x`, `0o`, or
/// `0b` radix prefixes that the alloy `ruint::Uint::FromStr` impl
/// would otherwise silently accept are rejected at the serde boundary,
/// so the cow JSON wire contract holds even when the value is fed
/// through serde rather than [`Amount::new`]. The asymmetry with the
/// lenient `Amount::new` constructor is deliberate per the
/// final-strategy AMENDMENTS document and `PROP-WS-008`.
#[test]
fn amount_rejects_hex_octal_binary_input_on_the_wire() {
    for forbidden in [
        r#""0x2a""#,
        r#""0X2a""#,
        r#""0o52""#,
        r#""0O52""#,
        r#""0b101010""#,
        r#""0B101010""#,
    ] {
        let outcome = serde_json::from_str::<Amount>(forbidden);
        assert!(
            outcome.is_err(),
            "Amount deserialize must reject radix-prefixed wire input `{forbidden}`"
        );
    }

    // The canonical decimal happy path stays accepted across the full
    // U256 range so the strict gate above does not over-fire.
    for canonical in ["\"0\"", "\"1\"", "\"1000000000000000000\""] {
        let amount: Amount = serde_json::from_str(canonical)
            .unwrap_or_else(|err| panic!("`{canonical}` must round-trip on the wire: {err}"));
        let json = serde_json::to_string(&amount).expect("Amount must serialize");
        assert_eq!(json, canonical);
    }
}

/// Stage B contract row (per AMENDMENTS §3.3): the cow `Amount::new`
/// constructor is intentionally **more lenient** than the `Deserialize`
/// impl so non-JSON callers (CLI flags, env vars, programmatic
/// inputs) can pass either a decimal string or a `0x`-prefixed hex
/// literal. The octal (`0o`) and binary (`0b`) prefixes that alloy's
/// `ruint::Uint::FromStr` four-radix sniffer accepts are still
/// rejected at the cow constructor boundary so the lenient surface
/// stays narrower than alloy's default.
#[test]
fn amount_constructor_accepts_decimal_and_hex_but_not_other_radix() {
    // Decimal accept.
    assert!(Amount::new("42").is_ok(), "decimal Amount::new must accept");
    assert!(
        Amount::new("00042").is_ok(),
        "leading-zero decimal Amount::new must accept"
    );
    // `0x`-prefixed hex accept (lowercase and uppercase X).
    assert_eq!(
        Amount::new("0x2a")
            .expect("0x-prefixed Amount::new must accept")
            .to_string(),
        "42"
    );
    assert_eq!(
        Amount::new("0X2a")
            .expect("0X-prefixed Amount::new must accept")
            .to_string(),
        "42"
    );
    // Octal and binary radix prefixes rejected.
    assert!(
        Amount::new("0o52").is_err(),
        "octal Amount::new must reject"
    );
    assert!(
        Amount::new("0O52").is_err(),
        "uppercase-O octal Amount::new must reject"
    );
    assert!(
        Amount::new("0b101010").is_err(),
        "binary Amount::new must reject"
    );
    assert!(
        Amount::new("0B101010").is_err(),
        "uppercase-B binary Amount::new must reject"
    );
    // Negatives, leading plus, and uint256 overflow are also rejected.
    assert!(
        Amount::new("-1").is_err(),
        "negative Amount::new must reject"
    );
    assert!(
        Amount::new("+1").is_err(),
        "leading-plus Amount::new must reject"
    );
    assert!(
        Amount::new(format!("0x1{}", "0".repeat(64))).is_err(),
        "2^256 Amount::new must reject as uint256 overflow"
    );
}

// ---- SignedAmount (decimal with optional leading minus) --------------

const SIGNED_AMOUNT_DECIMAL: &str = "-12345678901234567890";

#[test]
fn signed_amount_negative_round_trips_byte_identically_on_the_wire() {
    let amount =
        SignedAmount::new(SIGNED_AMOUNT_DECIMAL).expect("canonical SignedAmount must parse");
    assert_eq!(amount.to_string(), SIGNED_AMOUNT_DECIMAL);
    assert_eq!(amount.to_decimal_string(), SIGNED_AMOUNT_DECIMAL);

    let json = serde_json::to_string(&amount).expect("SignedAmount must serialize");
    assert_eq!(json, format!("\"{SIGNED_AMOUNT_DECIMAL}\""));

    let round_trip: SignedAmount =
        serde_json::from_str(&json).expect("SignedAmount must deserialize round trip");
    assert_eq!(round_trip.to_string(), SIGNED_AMOUNT_DECIMAL);
    assert_eq!(round_trip, amount);
    assert!(round_trip.is_negative());

    // Curated decimal-string boundary rows for both signs.
    for canonical in ["-1", "-12345", "0", "1"] {
        let parsed = SignedAmount::new(canonical)
            .unwrap_or_else(|err| panic!("canonical `{canonical}` must parse: {err}"));
        assert_eq!(parsed.to_string(), canonical);
        let json = serde_json::to_string(&parsed).expect("SignedAmount must serialize");
        assert_eq!(json, format!("\"{canonical}\""));
    }

    // `"-0"` canonicalises to `"0"` on the cow display surface (the
    // inner `I256` collapses the redundant sign byte before formatting).
    let neg_zero = SignedAmount::new("-0").expect("`-0` must parse");
    assert_eq!(neg_zero.to_string(), "0");

    // `I256::MIN` / `I256::MAX` survive a serde round-trip
    // byte-identically across the cow wire form.
    for boundary in [I256::MIN, I256::MAX] {
        let amount = SignedAmount::from_i256(boundary);
        let canonical = boundary.to_string();
        let json = serde_json::to_string(&amount).expect("SignedAmount must serialize");
        assert_eq!(json, format!("\"{canonical}\""));
        let round_trip: SignedAmount =
            serde_json::from_str(&json).expect("SignedAmount must deserialize round trip");
        assert_eq!(round_trip, amount);
    }
}

/// Stage B contract row: the cow `SignedAmount` accepts only the
/// signed-decimal shape `-?[0-9]+`. The `0x`-prefixed hex form, the
/// `-0x`-prefixed signed-hex form, and the leading-plus form are all
/// rejected at the constructor boundary and at the serde boundary
/// alike — `SignedAmount` is intentionally stricter than `Amount::new`
/// because the sign byte interacts with the radix-prefix grammar.
#[test]
fn signed_amount_rejects_hex_input_on_the_wire() {
    // Constructor-level rejections.
    for forbidden in ["0x5", "-0x5", "+5", "0X1", "0o52", "0b101"] {
        assert!(
            SignedAmount::new(forbidden).is_err(),
            "SignedAmount::new must reject `{forbidden}`"
        );
    }
    // Serde-boundary rejections (the cow `Deserialize` impl delegates
    // through the validating constructor so the two surfaces stay
    // aligned).
    for forbidden in [r#""0x5""#, r#""-0x5""#, r#""+5""#, r#""0X1""#] {
        let outcome = serde_json::from_str::<SignedAmount>(forbidden);
        assert!(
            outcome.is_err(),
            "SignedAmount deserialize must reject `{forbidden}`"
        );
    }
}

// ---- HexData (variable length) ----------------------------------------

#[test]
fn hex_data_wire_form_preserves_0x_prefix_across_variable_length() {
    let payload = HexData::new(HEX_DATA_FOUR_BYTES).expect("canonical HexData must parse");
    assert_eq!(payload.to_hex_string(), HEX_DATA_FOUR_BYTES);

    let json = serde_json::to_string(&payload).expect("HexData must serialize");
    assert_eq!(json, format!("\"{HEX_DATA_FOUR_BYTES}\""));

    let round_trip: HexData =
        serde_json::from_str(&json).expect("HexData must deserialize round trip");
    assert_eq!(round_trip.to_hex_string(), HEX_DATA_FOUR_BYTES);
}

#[test]
fn hex_data_empty_round_trips_with_bare_0x_prefix() {
    let empty = HexData::new(HEX_DATA_EMPTY).expect("empty HexData must parse");
    assert_eq!(empty.to_hex_string(), HEX_DATA_EMPTY);

    let json = serde_json::to_string(&empty).expect("empty HexData must serialize");
    assert_eq!(json, format!("\"{HEX_DATA_EMPTY}\""));
}

#[test]
fn hex_data_pads_odd_length_input_with_leading_zero_nibble() {
    let padded = HexData::new("0x123").expect("odd-length HexData must left-pad");
    assert_eq!(padded.to_hex_string(), "0x0123");
    assert_eq!(padded.byte_length(), 2);
}

// ---- OrderUid (56 bytes fixed) ----------------------------------------

#[test]
fn order_uid_wire_form_is_lowercase_0x_prefixed_114_chars() {
    let uid = OrderUid::new(ORDER_UID_HEX).expect("canonical OrderUid must parse");
    assert_eq!(uid.to_hex_string(), ORDER_UID_HEX);

    let json = serde_json::to_string(&uid).expect("OrderUid must serialize");
    assert_eq!(json, format!("\"{ORDER_UID_HEX}\""));

    let round_trip: OrderUid =
        serde_json::from_str(&json).expect("OrderUid must deserialize round trip");
    assert_eq!(round_trip.to_hex_string(), ORDER_UID_HEX);
}

#[test]
fn order_uid_rejects_malformed_inputs() {
    assert!(OrderUid::new("").is_err());
    assert!(OrderUid::new("not-hex").is_err());
    assert!(OrderUid::new("0x1234").is_err());
    assert!(serde_json::from_str::<OrderUid>(r#""0x""#).is_err());
}

// ---- R8 byte-parity property contract ---------------------------------

/// Asserts that the `write_into` zero-allocation accessor produces a
/// byte-identical string to the `to_hex_string` owned accessor across the
/// four byte-typed cow newtypes per AMENDMENTS §9.7.
#[test]
fn write_into_matches_to_hex_string_byte_identically() {
    let address = Address::new(ADDRESS_HEX).unwrap();
    let hash = Hash32::new(HASH32_HEX).unwrap();
    let uid = OrderUid::new(ORDER_UID_HEX).unwrap();
    let hex_data = HexData::new(HEX_DATA_FOUR_BYTES).unwrap();

    let mut buffer = String::new();

    address
        .write_into(&mut buffer)
        .expect("Address write_into must succeed");
    assert_eq!(buffer, address.to_hex_string());

    buffer.clear();
    hash.write_into(&mut buffer)
        .expect("Hash32 write_into must succeed");
    assert_eq!(buffer, hash.to_hex_string());

    buffer.clear();
    uid.write_into(&mut buffer)
        .expect("OrderUid write_into must succeed");
    assert_eq!(buffer, uid.to_hex_string());

    buffer.clear();
    hex_data
        .write_into(&mut buffer)
        .expect("HexData write_into must succeed");
    assert_eq!(buffer, hex_data.to_hex_string());

    // Test zero values
    buffer.clear();
    let zero_addr = Address::ZERO;
    zero_addr
        .write_into(&mut buffer)
        .expect("zero Address write_into must succeed");
    assert_eq!(buffer, zero_addr.to_hex_string());
}
