use alloy_primitives::U256;
use cow_sdk_core::{
    Address, Amount, Amounts, AppDataHash, BuyTokenDestination, Costs, FeeComponent, Hash32,
    HexData, NetworkFee, OrderData, OrderKind, OrderUid, QuoteAmountsAndCosts, SellTokenSource,
    ValidTo, ValidationError,
};

#[test]
fn shared_type_contract_matches_core_fixture() {
    let checksummed = Address::new("0x742D35CC6634C0532925A3B844BC9E7595F0BEBD").unwrap();
    let lowercase = Address::new("0x742d35cc6634c0532925a3b844bc9e7595f0bebd").unwrap();

    assert_eq!(
        checksummed.to_hex_string(),
        "0x742d35cc6634c0532925a3b844bc9e7595f0bebd"
    );
    assert_eq!(
        checksummed, lowercase,
        "Address PartialEq is case-insensitive across checksum casing"
    );
    assert_eq!(
        checksummed.to_hex_string().len(),
        42,
        "to_hex_string emits the canonical lowercase 0x-prefixed 42-character form"
    );
    assert_eq!(
        checksummed.as_slice().len(),
        20,
        "as_slice exposes the raw 20-byte representation"
    );
}

#[test]
fn canonical_order_and_quote_shapes_are_pinned() {
    // The canonical order field names and quote amount stages are pinned by
    // the serialized wire shape below rather than by re-asserting a constant
    // against a hand-typed copy of itself.
    let order = OrderData::new(
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        Amount::new("100").unwrap(),
        Amount::new("200").unwrap(),
        1_700_000_000,
        AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        Amount::new("5").unwrap(),
        OrderKind::Sell,
        true,
        SellTokenSource::External,
        BuyTokenDestination::Internal,
    );

    assert_eq!(order.sell_token_balance, SellTokenSource::External);
    assert_eq!(order.buy_token_balance, BuyTokenDestination::Internal);

    let json = serde_json::to_value(&order).unwrap();
    let object = json.as_object().unwrap();
    assert!(object.contains_key("sellToken"));
    assert!(object.contains_key("buyToken"));
    assert!(object.contains_key("receiver"));
    assert!(object.contains_key("appData"));
}

#[test]
fn omitted_receiver_resolves_to_the_pay_to_owner_sentinel() {
    // ADR 0061: an omitted `receiver` on the input boundary resolves to the zero
    // address — which the settlement contract reads as pay-to-owner — identically
    // to an explicit zero address. The invariant lives on `OrderData`'s serde
    // default (`#[serde(default = "default_order_receiver")]`).
    let without_receiver = r#"{
        "sellToken": "0x1111111111111111111111111111111111111111",
        "buyToken": "0x2222222222222222222222222222222222222222",
        "sellAmount": "100",
        "buyAmount": "200",
        "validTo": 1700000000,
        "appData": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "feeAmount": "5",
        "kind": "sell"
    }"#;
    let with_explicit_zero = without_receiver.replace(
        "\"sellToken\"",
        "\"receiver\": \"0x0000000000000000000000000000000000000000\", \"sellToken\"",
    );

    let omitted: OrderData = serde_json::from_str(without_receiver).unwrap();
    let explicit_zero: OrderData = serde_json::from_str(&with_explicit_zero).unwrap();

    assert_eq!(omitted.receiver, Address::ZERO);
    assert_eq!(
        omitted, explicit_zero,
        "an omitted receiver must deserialize identically to an explicit zero receiver",
    );
}

#[test]
fn a_concrete_receiver_is_distinct_from_the_pay_to_owner_sentinel() {
    let with_receiver = r#"{
        "receiver": "0x4444444444444444444444444444444444444444",
        "sellToken": "0x1111111111111111111111111111111111111111",
        "buyToken": "0x2222222222222222222222222222222222222222",
        "sellAmount": "100",
        "buyAmount": "200",
        "validTo": 1700000000,
        "appData": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "feeAmount": "5",
        "kind": "sell"
    }"#;
    let order: OrderData = serde_json::from_str(with_receiver).unwrap();
    assert_ne!(order.receiver, Address::ZERO);
}

#[test]
fn quote_amount_breakdown_serializes_canonical_stage_names() {
    let amounts = QuoteAmountsAndCosts::new(
        true,
        Costs::new(
            NetworkFee::new(Amount::new("1").unwrap(), Amount::new("2").unwrap()),
            FeeComponent::new(Amount::new("3").unwrap(), 4),
            FeeComponent::new(Amount::new("5").unwrap(), 6),
        ),
        Amounts::new(Amount::new("10").unwrap(), Amount::new("20").unwrap()),
        Amounts::new(Amount::new("11").unwrap(), Amount::new("21").unwrap()),
        Amounts::new(Amount::new("12").unwrap(), Amount::new("22").unwrap()),
        Amounts::new(Amount::new("13").unwrap(), Amount::new("23").unwrap()),
        Amounts::new(Amount::new("14").unwrap(), Amount::new("24").unwrap()),
        Amounts::new(Amount::new("15").unwrap(), Amount::new("25").unwrap()),
        Amounts::new(Amount::new("16").unwrap(), Amount::new("26").unwrap()),
    );
    let encoded = serde_json::to_value(amounts).unwrap();
    assert!(encoded.as_object().unwrap().contains_key("amountsToSign"));
}

#[test]
fn from_bytes_constructors_match_string_based_equivalents_byte_for_byte() {
    let address_bytes: [u8; 20] = [
        0x90, 0x08, 0xD1, 0x9f, 0x58, 0xAA, 0xBD, 0x9E, 0xD0, 0xD6, 0x09, 0x71, 0x56, 0x5A, 0xA8,
        0x51, 0x05, 0x60, 0xAB, 0x41,
    ];
    let from_bytes_address = Address::from_bytes(address_bytes);
    let from_new_address =
        Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").expect("valid address literal");
    assert_eq!(
        from_bytes_address, from_new_address,
        "Address::from_bytes must match the case-insensitive Address::new equivalent"
    );
    assert_eq!(
        from_bytes_address.to_hex_string(),
        "0x9008d19f58aabd9ed0d60971565aa8510560ab41",
        "Address::from_bytes must produce the canonical lowercase hex form"
    );

    let hash_bytes: [u8; 32] = [0xab; 32];
    let from_bytes_hash = Hash32::from_bytes(hash_bytes);
    let from_new_hash = Hash32::new(format!("0x{}", "ab".repeat(32))).expect("valid hash literal");
    assert_eq!(
        from_bytes_hash, from_new_hash,
        "Hash32::from_bytes must match the Hash32::new equivalent"
    );

    let app_data_bytes: [u8; 32] = [0x5a; 32];
    let from_bytes_app_data = AppDataHash::from_bytes(app_data_bytes);
    let from_new_app_data =
        AppDataHash::new(format!("0x{}", "5a".repeat(32))).expect("valid app-data hash literal");
    assert_eq!(
        from_bytes_app_data, from_new_app_data,
        "AppDataHash::from_bytes must match the AppDataHash::new equivalent"
    );

    let mut uid_bytes = [0u8; 56];
    for (i, byte) in uid_bytes.iter_mut().enumerate() {
        *byte = u8::try_from(i).expect("index fits in u8 for the 56-byte test array");
    }
    let from_bytes_uid = OrderUid::from_bytes(uid_bytes);
    let mut hex_form = String::with_capacity(uid_bytes.len() * 2);
    for byte in uid_bytes {
        use std::fmt::Write as _;
        write!(&mut hex_form, "{byte:02x}").expect("writing to a String never fails");
    }
    let from_new_uid =
        OrderUid::new(format!("0x{hex_form}")).expect("valid order UID literal for fixture");
    assert_eq!(
        from_bytes_uid, from_new_uid,
        "OrderUid::from_bytes must match the OrderUid::new equivalent"
    );
}

#[test]
fn typed_amount_exposes_semantic_accessors() {
    let amount = Amount::from_u256(U256::from(1_000_000_000_000_000_000u128));
    assert_eq!(amount.to_string(), "1000000000000000000");
    assert_eq!(amount.as_u256(), &U256::from(1_000_000_000_000_000_000u128));

    let parsed: Amount = "1000000000000000000".try_into().unwrap();
    assert_eq!(parsed, amount);
}

#[test]
fn amount_checked_arithmetic_preserves_option_shape() {
    let small = Amount::from(7u32);
    let large = Amount::from(11u32);
    let factor = Amount::from(3u32);

    assert_eq!(
        small.checked_add(large),
        Some(Amount::from(18u32)),
        "checked_add must return Some for in-range U256 inputs"
    );
    assert_eq!(
        small.checked_sub(large),
        None,
        "checked_sub must expose underflow through the Option boundary",
    );
    assert_eq!(
        small.saturating_sub(large),
        Amount::ZERO,
        "saturating_sub must clamp underflow to zero instead of wrapping",
    );
    assert_eq!(
        large.checked_mul(factor),
        Some(Amount::from(33u32)),
        "checked_mul must return Some for in-range U256 inputs"
    );
}

#[test]
fn valid_to_absolute_accepts_any_representable_epoch() {
    let absolute = ValidTo::absolute(1_800_000_000);
    assert_eq!(absolute.as_u32(), 1_800_000_000);
    assert_eq!(absolute.as_u64(), 1_800_000_000);

    let converted: u32 = absolute.into();
    assert_eq!(converted, 1_800_000_000);

    let via_from: ValidTo = 2_000_000_000u32.into();
    assert_eq!(via_from, ValidTo::absolute(2_000_000_000));
}

#[test]
fn valid_to_relative_accepts_any_duration_within_the_u32_ceiling() {
    let now = 1_800_000_000u64;

    // No operator-tunable window: a short duration the old 30s floor would have
    // rejected is accepted, and a long one the old 90-day ceiling would have
    // rejected is accepted too, as long as the absolute timestamp fits `u32`.
    let short = ValidTo::relative(now, 5).expect("a short duration is accepted");
    assert_eq!(short.as_u64(), now + 5);

    let long_duration = 200 * 24 * 60 * 60;
    let long = ValidTo::relative(now, long_duration).expect("a long duration is accepted");
    assert_eq!(long.as_u64(), now + long_duration);

    // The only fail-closed boundary is the protocol-fixed `u32` epoch ceiling.
    match ValidTo::relative(u64::from(u32::MAX), 1) {
        Err(cow_sdk_core::CoreError::Validation(ValidationError::ValidToOutOfRange {
            actual_seconds,
        })) => assert_eq!(actual_seconds, u64::from(u32::MAX) + 1),
        other => panic!("a timestamp past the u32 ceiling must fail closed, got {other:?}"),
    }
}

#[test]
fn typed_primitives_normalize_and_fail_closed() {
    assert_eq!(Amount::new("00042").unwrap().to_string(), "42");
    assert_eq!(Amount::new("0x2a").unwrap().to_string(), "42");
    assert!(Amount::new("-1").is_err());
    assert!(Amount::new("abc").is_err());
    assert!(Amount::new(format!("0x1{}", "0".repeat(64))).is_err());

    assert_eq!(HexData::new("0xabc").unwrap().to_hex_string(), "0x0abc");
    assert_eq!(HexData::empty().to_hex_string(), "0x");
    assert!(HexData::new("1234").is_err());

    let hash = Hash32::new(format!("0x{}", "ab".repeat(32))).unwrap();
    assert_eq!(hash.to_hex_string().len(), 66);
    assert!(Hash32::new("0x1234").is_err());
}

#[test]
fn app_data_hash_from_full_app_data_matches_keccak256_of_bytes() {
    // Lock the byte-canonical invariant: AppDataHash::from_full_app_data
    // must equal keccak256 over the input bytes on every supported platform.
    let body = "{\"appCode\":\"cow-rs\",\"metadata\":{},\"version\":\"1.6.0\"}";
    let computed = AppDataHash::from_full_app_data(body);
    let expected = AppDataHash::from_bytes(*alloy_primitives::keccak256(body.as_bytes()));
    assert_eq!(computed, expected);

    // Empty-document corner case: keccak256("{}") is non-zero by construction.
    let empty_doc = AppDataHash::from_full_app_data("{}");
    assert!(!empty_doc.is_zero());
    assert_eq!(
        empty_doc,
        AppDataHash::from_bytes(*alloy_primitives::keccak256(b"{}")),
    );
}

#[test]
fn app_data_hash_from_full_app_data_is_deterministic() {
    let body = "{\"appCode\":\"cow-rs\",\"metadata\":{},\"version\":\"1.6.0\"}";
    let first = AppDataHash::from_full_app_data(body);
    let second = AppDataHash::from_full_app_data(body);
    assert_eq!(first, second);
}

#[test]
fn app_data_hash_from_full_app_data_distinguishes_byte_distinct_inputs() {
    // Two semantically-equal JSON documents that differ only in object key
    // order produce different byte sequences and therefore different
    // digests. This pins the byte-canonical contract documented on the
    // helper: the caller is responsible for canonicalising before hashing.
    let ordered = "{\"a\":1,\"b\":2}";
    let reordered = "{\"b\":2,\"a\":1}";
    assert_ne!(
        AppDataHash::from_full_app_data(ordered),
        AppDataHash::from_full_app_data(reordered)
    );
}

#[test]
fn cow_primitive_newtype_zero_constants_equal_alloy_zero() {
    // Every cow primitive newtype that carries a canonical zero ships
    // `pub const ZERO: Self`. The constant must equal the value the
    // typed constructor produces from the canonical zero string and
    // must report `is_zero()` true.

    // Address (20 zero bytes)
    let zero_address = Address::new("0x0000000000000000000000000000000000000000").unwrap();
    assert_eq!(Address::ZERO, zero_address);
    assert!(Address::ZERO.is_zero());

    // Amount (uint256 zero)
    let zero_amount = Amount::new("0").unwrap();
    assert_eq!(Amount::ZERO, zero_amount);
    assert!(Amount::ZERO.is_zero());

    // AppDataHash (32 zero bytes)
    let zero_app_data_hash =
        AppDataHash::new("0x0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap();
    assert_eq!(AppDataHash::ZERO, zero_app_data_hash);
    assert!(AppDataHash::ZERO.is_zero());

    // Hash32 (32 zero bytes)
    let zero_hash32 =
        Hash32::new("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    assert_eq!(Hash32::ZERO, zero_hash32);
    assert!(Hash32::ZERO.is_zero());

    // OrderUid (56 zero bytes)
    let zero_uid_hex = format!("0x{}", "00".repeat(56));
    let zero_uid = OrderUid::new(&zero_uid_hex).unwrap();
    assert_eq!(OrderUid::ZERO, zero_uid);
    assert!(OrderUid::ZERO.is_zero());
}

// Contract tests for the narrowed checked / saturating / bit_len /
// MAX / MIN arithmetic surface on `Amount`.
// The newtype exposes no bare `+` `-` `*` operators and no `pow`; the
// only arithmetic is fallible-by-return (`checked_*` -> `Option`) or
// an explicit `saturating_*` clamp, so overflow can never silently
// wrap or panic. `checked_pow` is the genuine overflow-detecting
// variant (the inner `ruint::Uint` / `alloy` `pow` is wrapping),
// which the `*_checked_pow_returns_none_on_overflow` tests pin.

#[test]
fn amount_max_constant_equals_alloy_u256_max() {
    assert_eq!(Amount::MAX.into_u256(), U256::MAX);
    assert!(Amount::MAX > Amount::ZERO);
}

#[test]
fn amount_parse_units_is_exact_and_rejects_bad_input() {
    use cow_sdk_core::CoreError;

    // Exact integer scaling: the decimal string is multiplied by
    // 10^decimals using integer arithmetic, never floating point, so
    // every digit is preserved byte-for-byte.
    assert_eq!(
        Amount::parse_units("1", 18).unwrap(),
        Amount::from(1_000_000_000_000_000_000u128),
        "parse_units(\"1\", 18) must scale by 10^18 exactly",
    );
    assert_eq!(
        Amount::parse_units("1000", 6).unwrap(),
        Amount::from(1_000_000_000u128),
        "parse_units(\"1000\", 6) must scale by 10^6 exactly",
    );
    assert_eq!(
        Amount::parse_units("1.5", 18).unwrap(),
        Amount::from(1_500_000_000_000_000_000u128),
        "parse_units must scale the fractional part exactly with no f64 drift",
    );

    // Fractional digits beyond `decimals` are truncated, matching the
    // orderbook atomic-unit contract: ".1234" at 3 decimals keeps the
    // first three fractional digits ("123") and drops the "4".
    assert_eq!(
        Amount::parse_units(".1234", 3).unwrap(),
        Amount::from(123u128),
        "parse_units must truncate fractional digits beyond `decimals`",
    );

    // Surrounding whitespace is trimmed before parsing, so a
    // whitespace-padded integer at zero decimals parses to the bare
    // integer.
    assert_eq!(
        Amount::parse_units("  2  ", 0).unwrap(),
        Amount::from(2u128),
        "parse_units must trim surrounding whitespace before parsing",
    );

    // Fail-closed rejections. Empty and whitespace-only inputs surface
    // `EmptyField` (NOT alloy's fail-open `Ok(0)`).
    assert!(matches!(
        Amount::parse_units("", 18),
        Err(CoreError::Validation(ValidationError::EmptyField {
            field: "amount"
        })),
    ));
    assert!(matches!(
        Amount::parse_units("   ", 18),
        Err(CoreError::Validation(ValidationError::EmptyField {
            field: "amount"
        })),
    ));

    // A leading sign is rejected as `InvalidNumeric` (a leading `-`
    // must NOT route to alloy's signed arm and silently widen into a
    // huge positive via two's-complement).
    assert!(matches!(
        Amount::parse_units("-1", 18),
        Err(CoreError::Validation(ValidationError::InvalidNumeric {
            field: "amount"
        })),
    ));
    assert!(matches!(
        Amount::parse_units("+1", 18),
        Err(CoreError::Validation(ValidationError::InvalidNumeric {
            field: "amount"
        })),
    ));

    // Non-numeric input is rejected as `InvalidNumeric`.
    assert!(matches!(
        Amount::parse_units("abc", 18),
        Err(CoreError::Validation(ValidationError::InvalidNumeric {
            field: "amount"
        })),
    ));

    // `decimals` above 77 (the `alloy_primitives::utils::Unit::MAX`
    // bound) is rejected at construction time with the kept
    // `DecimalsOutOfRange` error carrying the rejected scale and the
    // documented maximum.
    assert!(matches!(
        Amount::parse_units("1", 78),
        Err(CoreError::Validation(ValidationError::DecimalsOutOfRange {
            actual: 78,
            max: 77,
        })),
    ));
    // The gate scales beyond the immediate 78 case: u8::MAX is also
    // rejected with its own actual value.
    assert!(matches!(
        Amount::parse_units("1", u8::MAX),
        Err(CoreError::Validation(ValidationError::DecimalsOutOfRange {
            actual: 255,
            max: 77,
        })),
    ));
}

#[test]
fn amount_format_units_preserves_trailing_zeros() {
    // Pins the trailing-zero preservation contract across canonical
    // and edge-case `(atoms, decimals)` shapes. The load-bearing
    // contrast is the canonical 1-ether row (atoms = 10^18, decimals
    // = 18): cow renders this as the full 18-digit fractional
    // expansion, while the JavaScript ecosystem's `formatUnits`
    // helper renders it as `"1.0"`. This divergence is the documented
    // public-contract reason for the cow surface, so the row is
    // required for any future refactor that touches the formatter.

    // Row 1: zero atoms, zero decimals → bare integer "0" form.
    assert_eq!(Amount::from_u256(U256::ZERO).format_units(0), "0");

    // Row 2: small atoms, zero decimals → bare integer, no padding.
    assert_eq!(Amount::from_u256(U256::from(42u8)).format_units(0), "42");

    // Row 3: zero atoms, canonical 18 decimals → "0" plus 18 zeros.
    assert_eq!(
        Amount::from_u256(U256::ZERO).format_units(18),
        "0.000000000000000000",
    );

    // Row 4: smallest non-zero atoms, 18 decimals → zero integer
    // part with the documented zero-padded fractional substring.
    assert_eq!(
        Amount::from_u256(U256::from(1u8)).format_units(18),
        "0.000000000000000001",
    );

    // Row 5: (atoms=10, decimals=1) is the boundary case where the
    // cow output coincidentally matches `formatUnits` (both produce
    // "1.0"). This case pins that the cow format does not over-trim.
    assert_eq!(Amount::from_u256(U256::from(10u8)).format_units(1), "1.0");

    // Row 6: canonical 1-ether case — the load-bearing trim contrast.
    // cow preserves the full 18-digit fractional expansion; the
    // ethers/viem/services `formatUnits` helper would trim to "1.0".
    assert_eq!(
        Amount::from_u256(U256::from(1_000_000_000_000_000_000u128)).format_units(18),
        "1.000000000000000000",
    );

    // Row 7: rounded value (atoms=100, decimals=2) → preserved
    // trailing zeros (cow: "1.00", JavaScript `formatUnits` would
    // produce "1.0").
    assert_eq!(Amount::from_u256(U256::from(100u8)).format_units(2), "1.00");

    // Row 8: maximum representable decimals (77). One atom at 77
    // decimals → integer "0" plus 76 zeros plus a trailing "1".
    let r8 = Amount::from_u256(U256::from(1u8)).format_units(77);
    let expected_r8 = format!("0.{}{}", "0".repeat(76), "1");
    assert_eq!(r8, expected_r8);
    assert_eq!(
        r8.len(),
        "0.".len() + 77,
        "decimals == 77 must yield a 77-character fractional substring",
    );

    // Row 9: `(U256::MAX, 18)` sentinel — pins the wire-byte contract
    // at the largest representable unsigned integer with the
    // canonical 18-decimal token scale. The integer part is the full
    // 60-digit `2^256 - 1 / 10^18` quotient and the fractional part
    // is the remaining 18 atoms-of-the-quotient padded to length 18.
    assert_eq!(
        Amount::from_u256(U256::MAX).format_units(18),
        "115792089237316195423570985008687907853269984665640564039457.584007913129639935",
    );

    // Row 10: `(U256::MAX, 77)` sentinel — pins the wire-byte contract
    // at the largest representable unsigned integer paired with the
    // maximum supported decimals scale. The integer part is the
    // single digit `1` and the fractional part is the remaining 77
    // digits of `U256::MAX` with no padding.
    let r10 = Amount::from_u256(U256::MAX).format_units(77);
    assert_eq!(
        r10,
        "1.15792089237316195423570985008687907853269984665640564039457584007913129639935",
    );
    assert_eq!(
        r10.len(),
        "1.".len() + 77,
        "decimals == 77 must yield a 77-character fractional substring",
    );

    // parse ∘ format round-trip: format_units preserves the full
    // fractional width precisely so the output re-parses back to the
    // originating atoms for a representative non-trivial value.
    let representative = Amount::from_u256(U256::from(1_500_000_000_000_000_000u128));
    assert_eq!(
        Amount::parse_units(representative.format_units(18), 18).unwrap(),
        representative,
        "parse_units(format_units(x, 18), 18) must round-trip back to x",
    );
}

#[test]
fn amount_from_units_is_exact_and_agrees_with_parse_units() {
    use cow_sdk_core::CoreError;

    // Exact integer scaling: the whole-unit count is multiplied by
    // 10^decimals with checked integer arithmetic (no string round-trip,
    // no floating point), so every digit is preserved.
    assert_eq!(
        Amount::from_units(1, 18).unwrap(),
        Amount::from(1_000_000_000_000_000_000u128),
        "from_units(1, 18) must scale by 10^18 exactly",
    );
    assert_eq!(
        Amount::from_units(1000, 6).unwrap(),
        Amount::from(1_000_000_000u128),
        "from_units(1000, 6) must scale by 10^6 exactly",
    );

    // `from_units` is the numeric door to the same atomic value
    // `parse_units` produces for the equivalent whole number: the two
    // must agree for every whole-number input.
    for (whole, decimals) in [(0u128, 18), (1, 18), (1000, 6), (7, 8), (123_456, 0)] {
        assert_eq!(
            Amount::from_units(whole, decimals).unwrap(),
            Amount::parse_units(whole.to_string(), decimals).unwrap(),
            "from_units({whole}, {decimals}) must equal parse_units of the same whole number",
        );
    }

    // Zero whole units is the zero amount; `decimals == 0` does not scale.
    assert_eq!(
        Amount::from_units(0, 18).unwrap(),
        Amount::from(0u128),
        "from_units(0, _) must be the zero amount",
    );
    assert_eq!(
        Amount::from_units(42, 0).unwrap(),
        Amount::from(42u128),
        "from_units(_, 0) must not scale",
    );

    // `decimals` above 77 (the `alloy_primitives::utils::Unit::MAX` bound)
    // is rejected with the same `DecimalsOutOfRange` error `parse_units`
    // uses, carrying the rejected scale and the documented maximum.
    assert!(matches!(
        Amount::from_units(1, 78),
        Err(CoreError::Validation(ValidationError::DecimalsOutOfRange {
            actual: 78,
            max: 77,
        })),
    ));
    assert!(matches!(
        Amount::from_units(1, u8::MAX),
        Err(CoreError::Validation(ValidationError::DecimalsOutOfRange {
            actual: 255,
            max: 77,
        })),
    ));

    // A whole-unit count whose scaled magnitude exceeds `uint256` fails
    // closed with `NumericOverflow` instead of wrapping: `u128::MAX` whole
    // units at 77 decimals is far beyond `U256::MAX`.
    assert!(matches!(
        Amount::from_units(u128::MAX, 77),
        Err(CoreError::Validation(ValidationError::NumericOverflow {
            field: "amount"
        })),
    ));
}
