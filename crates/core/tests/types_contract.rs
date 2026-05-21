use alloy_primitives::{I256, U256};
use cow_sdk_core::{
    Address, Amount, Amounts, AppDataHash, AppDataHex, BuyTokenDestination, Costs, DecimalAmount,
    FeeComponent, Hash32, HexData, NetworkFee, ORDER_TYPE_FIELD_NAMES, OrderKind, OrderUid,
    QUOTE_AMOUNT_STAGE_NAMES, QuoteAmountsAndCosts, SellTokenSource, SignedAmount, UnsignedOrder,
    VALID_TO_MAX_RELATIVE_SECONDS, VALID_TO_MIN_RELATIVE_SECONDS, ValidTo, ValidationError,
    addresses_equal, token_id,
};

fn core_fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../parity/fixtures/core.json"))
        .expect("core fixture must remain valid json")
}

#[test]
fn shared_type_contract_matches_core_fixture() {
    let fixture = core_fixture();
    assert_eq!(fixture["surface"].as_str().unwrap(), "core");

    let address_case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "core-evm-address-contract")
        .unwrap();
    assert_eq!(
        address_case["expected"]["address_prefix"].as_str().unwrap(),
        "0x"
    );
    assert_eq!(address_case["expected"]["hex_chars"].as_u64().unwrap(), 40);

    let checksummed = Address::new("0x742D35CC6634C0532925A3B844BC9E7595F0BEBD").unwrap();
    let lowercase = Address::new("0x742d35cc6634c0532925a3b844bc9e7595f0bebd").unwrap();

    assert_eq!(
        checksummed.normalized_key(),
        "0x742d35cc6634c0532925a3b844bc9e7595f0bebd"
    );
    assert!(addresses_equal(&checksummed, &lowercase));
    assert_eq!(
        checksummed, lowercase,
        "PartialEq must agree with addresses_equal on case variants"
    );
    assert_eq!(
        checksummed.byte_length(),
        20,
        "byte_length must match the fixed EVM address width"
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

    let token_case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "core-token-identity-contract")
        .unwrap();
    assert_eq!(
        token_id(1, &checksummed),
        token_case["expected"]["token_id"].as_str().unwrap()
    );
}

#[test]
fn canonical_order_and_quote_shapes_are_pinned() {
    let fixture = core_fixture();
    let shared_case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "core-shared-order-and-quote-surfaces")
        .unwrap();

    let expected_fields: Vec<&str> = shared_case["expected"]["order_fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    let expected_stages: Vec<&str> = shared_case["expected"]["quote_amount_stages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();

    assert_eq!(ORDER_TYPE_FIELD_NAMES.to_vec(), expected_fields);
    assert_eq!(QUOTE_AMOUNT_STAGE_NAMES.to_vec(), expected_stages);

    let order = UnsignedOrder::new(
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        Amount::new("100").unwrap(),
        Amount::new("200").unwrap(),
        1_700_000_000,
        AppDataHex::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
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
    let from_bytes_app_data = AppDataHex::from_bytes(app_data_bytes);
    let from_new_app_data =
        AppDataHex::new(format!("0x{}", "5a".repeat(32))).expect("valid app-data hash literal");
    assert_eq!(
        from_bytes_app_data, from_new_app_data,
        "AppDataHex::from_bytes must match the AppDataHex::new equivalent"
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
fn typed_amount_and_decimal_amount_expose_semantic_accessors() {
    let amount = Amount::from_u256(U256::from(1_000_000_000_000_000_000u128));
    assert_eq!(amount.to_string(), "1000000000000000000");
    assert_eq!(amount.as_u256(), &U256::from(1_000_000_000_000_000_000u128));

    let parsed: Amount = "1000000000000000000".try_into().unwrap();
    assert_eq!(parsed, amount);

    let decimal = DecimalAmount::new(U256::from(1_000_000_000_000_000_000u128), 18);
    assert_eq!(decimal.decimals(), 18);
    assert_eq!(decimal.atoms(), &U256::from(1_000_000_000_000_000_000u128));
    assert!((decimal.to_f64_approx() - 1.0).abs() < 1e-12);

    let clamped = DecimalAmount::from_whole_approx(-0.5, 18);
    assert_eq!(clamped.atoms(), &U256::ZERO);
}

#[test]
fn amount_addition_is_commutative_across_curated_boundaries() {
    // The fifth boundary value (`U256::MAX - 1`) is the largest value that
    // can still participate in a non-overflowing pairwise sum with
    // `Amount::from(1u32)`. Combined with `Amount::from(u128::MAX)`,
    // `u64::MAX`, `1u32`, and zero this exercises the full `U256` storage
    // range without crossing the `uint256` ceiling.
    let boundaries = [
        Amount::ZERO,
        Amount::from(1u32),
        Amount::from(u64::MAX),
        Amount::from(u128::MAX),
        Amount::from_u256(U256::MAX - U256::from(u128::MAX) - U256::from(1u8)),
    ];

    for &left in &boundaries {
        for &right in &boundaries {
            assert_eq!(
                left + right,
                right + left,
                "Amount addition must be commutative for {left} and {right}"
            );
        }
    }
}

#[test]
fn amount_addition_is_associative_for_curated_triple() {
    let a = Amount::from_u256((U256::from(1u8) << 128usize) + U256::from(7u32));
    let b = Amount::from_u256((U256::from(1u8) << 192usize) + U256::from(11u32));
    // `c` deliberately stays below `1 << 255` so the curated triple sum
    // (~`1 << 255 + 1 << 192 + 1 << 128`) does not exceed `U256::MAX`.
    let c = Amount::from_u256((U256::from(1u8) << 254usize) + U256::from(13u32));

    assert_eq!(
        (a + b) + c,
        a + (b + c),
        "Amount addition must delegate to associative U256 addition"
    );
}

#[test]
fn amount_checked_arithmetic_preserves_option_shape() {
    let small = Amount::from(7u32);
    let large = Amount::from(11u32);
    let factor = Amount::from(3u32);

    assert_eq!(
        small.checked_add(&large),
        Some(Amount::from(18u32)),
        "checked_add must return Some for in-range U256 inputs"
    );
    assert_eq!(
        large - small,
        Amount::from(4u32),
        "Sub must delegate to the inner U256 and return the typed difference",
    );
    assert_eq!(
        small.checked_sub(&large),
        None,
        "checked_sub must expose underflow through the Option boundary",
    );
    assert_eq!(
        small.saturating_sub(&large),
        Amount::ZERO,
        "saturating_sub must clamp underflow to zero instead of wrapping",
    );
    assert_eq!(
        large.checked_mul(&factor),
        Some(Amount::from(33u32)),
        "checked_mul must return Some for in-range U256 inputs"
    );

    let mut running = small;
    running += large;
    assert_eq!(running, Amount::from(18u32));
    running -= Amount::from(8u32);
    assert_eq!(running, Amount::from(10u32));
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
fn valid_to_relative_rejects_values_outside_the_supported_window() {
    let now = 1_800_000_000u64;

    let min = VALID_TO_MIN_RELATIVE_SECONDS;
    let max = VALID_TO_MAX_RELATIVE_SECONDS;

    let at_min = ValidTo::relative(now, u64::from(min)).expect("min must be accepted");
    assert_eq!(at_min.as_u64(), now + u64::from(min));

    let at_max = ValidTo::relative(now, u64::from(max)).expect("max must be accepted");
    assert_eq!(
        at_max.as_u64(),
        (now + u64::from(max)).min(u64::from(u32::MAX))
    );

    match ValidTo::relative(now, u64::from(min) - 1) {
        Err(err) => {
            let validation: ValidationError = match err {
                cow_sdk_core::CoreError::Validation(v) => v,
                other => panic!("expected validation error, got {other:?}"),
            };
            assert!(matches!(
                validation,
                ValidationError::ValidToOutOfRange { .. }
            ));
        }
        Ok(value) => panic!("sub-minimum duration must fail closed, got {value:?}"),
    }

    match ValidTo::relative(now, u64::from(max) + 1) {
        Err(err) => {
            assert!(matches!(
                err,
                cow_sdk_core::CoreError::Validation(ValidationError::ValidToOutOfRange { .. })
            ));
        }
        Ok(value) => panic!("above-maximum duration must fail closed, got {value:?}"),
    }
}

#[test]
fn typed_primitives_normalize_and_fail_closed() {
    assert_eq!(Amount::new("00042").unwrap().to_string(), "42");
    assert_eq!(Amount::new("0x2a").unwrap().to_string(), "42");
    assert!(Amount::new("-1").is_err());
    assert!(Amount::new("abc").is_err());
    assert!(Amount::new(format!("0x1{}", "0".repeat(64))).is_err());

    assert_eq!(SignedAmount::new("-0005").unwrap().to_string(), "-5");
    assert_eq!(SignedAmount::new("0").unwrap().to_string(), "0");
    assert!(SignedAmount::new("0x5").is_err());

    assert_eq!(HexData::new("0xabc").unwrap().to_hex_string(), "0x0abc");
    assert_eq!(HexData::empty().to_hex_string(), "0x");
    assert!(HexData::new("1234").is_err());

    let hash = Hash32::new(format!("0x{}", "ab".repeat(32))).unwrap();
    assert_eq!(hash.to_hex_string().len(), 66);
    assert!(Hash32::new("0x1234").is_err());
}

#[test]
fn signed_amount_typed_accessors_preserve_i256_storage() {
    // Largest representable positive `I256` value: `2^255 - 1`. The cow
    // newtype storage is `#[repr(transparent)]` over `I256`, so every
    // bit pattern fits in 32 bytes and the accessor surface returns
    // borrowed / owned `I256` views without intermediate parsing.
    let value = I256::MAX;
    let canonical = value.to_string();
    let amount = SignedAmount::from_i256(value);

    assert_eq!(amount.as_i256(), &value);
    assert_eq!(amount.to_string(), canonical);
    assert_eq!(amount.into_i256(), value);
}

#[test]
fn signed_amount_add_and_sub_delegate_to_i256() {
    let a = SignedAmount::new("7").unwrap();
    let b = SignedAmount::new("-3").unwrap();
    let c = SignedAmount::new("12").unwrap();

    assert_eq!(a + b, SignedAmount::new("4").unwrap());
    assert_eq!(b + a, SignedAmount::new("4").unwrap());
    assert_eq!((a + b) + c, a + (b + c));
    assert_eq!(a + SignedAmount::ZERO, SignedAmount::new("7").unwrap());
    assert_eq!(a - a, SignedAmount::ZERO);

    let mut total = a;
    total += b;
    assert_eq!(total, SignedAmount::new("4").unwrap());

    total -= c;
    assert_eq!(total, SignedAmount::new("-8").unwrap());
}

#[test]
fn signed_amount_checked_arithmetic_returns_i256_results() {
    let lhs = SignedAmount::new("-12345678901234567890").unwrap();
    let rhs = SignedAmount::new("9876543210").unwrap();
    // A 10^9 multiplier keeps `rhs * multiplier` well inside the `I256`
    // representable range while still exercising the full 64-bit-class
    // arithmetic surface.
    let multiplier = SignedAmount::from_i256(I256::try_from(1_000_000_000i64).unwrap());

    let sum = lhs.checked_add(&rhs).unwrap();
    assert_eq!(
        sum.into_i256(),
        lhs.as_i256().checked_add(*rhs.as_i256()).unwrap()
    );

    let difference = lhs.checked_sub(&rhs).unwrap();
    assert_eq!(
        difference.into_i256(),
        lhs.as_i256().checked_sub(*rhs.as_i256()).unwrap()
    );

    let product = rhs.checked_mul(&multiplier).unwrap();
    assert_eq!(
        product.into_i256(),
        rhs.as_i256().checked_mul(*multiplier.as_i256()).unwrap()
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

    // SignedAmount (int256 zero)
    let zero_signed = SignedAmount::new("0").unwrap();
    assert_eq!(SignedAmount::ZERO, zero_signed);
    assert!(SignedAmount::ZERO.is_zero());

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
