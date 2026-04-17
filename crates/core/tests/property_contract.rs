#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::derive_partial_eq_without_eq,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, style, and perf lints acceptable in test helper code"
)]

use num_bigint::BigUint;

use cow_sdk_core::{
    Address, Amount, AppDataHex, ChainId, Hash32, HexData, OrderUid, SupportedChainId,
    addresses_equal, token_id,
};

const CASE_COUNT: u64 = 128;

#[derive(Clone)]
struct CaseRng {
    state: u64,
}

impl CaseRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(0x9E37_79B9_7F4A_7C15),
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.state = value;
        value.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn fill<const N: usize>(&mut self) -> [u8; N] {
        let mut bytes = [0u8; N];
        for byte in &mut bytes {
            *byte = self.next_u64() as u8;
        }
        bytes
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }

    fn supported_chain(&mut self) -> SupportedChainId {
        SupportedChainId::ALL[(self.next_u64() as usize) % SupportedChainId::ALL.len()]
    }

    fn decimal_amount_components(&mut self) -> (BigUint, String, String) {
        let bytes = self.fill::<32>();
        let value = BigUint::from_bytes_be(&bytes);
        let canonical = value.to_str_radix(10);
        let hex_form = format!("0x{}", value.to_str_radix(16));
        (value, canonical, hex_form)
    }
}

fn mixed_case_hex(rng: &mut CaseRng, bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2 + 2);
    output.push_str("0x");
    for byte in bytes {
        let hi = (byte >> 4) & 0x0F;
        let lo = byte & 0x0F;
        let hi_char = hex_nibble(hi, rng.next_bool());
        let lo_char = hex_nibble(lo, rng.next_bool());
        output.push(hi_char);
        output.push(lo_char);
    }
    output
}

fn hex_nibble(value: u8, uppercase: bool) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => {
            if uppercase {
                (b'A' + value - 10) as char
            } else {
                (b'a' + value - 10) as char
            }
        }
        _ => unreachable!("nibble value must fit in four bits"),
    }
}

#[test]
fn address_roundtrip_preserves_input_case_across_seeds() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed);
        let bytes = rng.fill::<20>();
        let mixed = mixed_case_hex(&mut rng, &bytes);

        let address = Address::new(&mixed).unwrap();
        assert_eq!(
            address.as_str(),
            mixed,
            "address must preserve the original input string exactly"
        );

        let roundtrip: String = address.clone().into();
        assert_eq!(
            roundtrip, mixed,
            "address-to-string conversion must return the stored input"
        );

        let rebuilt = Address::new(roundtrip).unwrap();
        assert_eq!(
            rebuilt, address,
            "rebuilding an address from its own string form must produce an equal value"
        );
    }
}

#[test]
fn address_normalized_key_is_lowercase_case_insensitive() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed);
        let bytes = rng.fill::<20>();
        let mixed = mixed_case_hex(&mut rng, &bytes);
        let lowercase = format!("0x{}", hex::encode(bytes));
        let uppercase = format!("0x{}", hex::encode_upper(bytes));

        let address_mixed = Address::new(&mixed).unwrap();
        let address_lower = Address::new(&lowercase).unwrap();
        let address_upper = Address::new(&uppercase).unwrap();

        assert_eq!(
            address_mixed.normalized_key(),
            lowercase,
            "normalized key must always be the lowercase form"
        );
        assert_eq!(
            address_lower.normalized_key(),
            address_upper.normalized_key(),
            "case variants must share one normalized key"
        );
        assert!(
            addresses_equal(&address_mixed, &address_lower),
            "addresses_equal must treat case variants as equal"
        );
        assert!(
            addresses_equal(&address_upper, &address_lower),
            "addresses_equal must treat uppercase and lowercase as equal"
        );
    }
}

#[test]
fn address_partial_eq_and_hash_are_case_insensitive() {
    use std::collections::{HashMap, HashSet};

    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed);
        let bytes = rng.fill::<20>();
        let mixed = mixed_case_hex(&mut rng, &bytes);
        let lowercase = format!("0x{}", hex::encode(bytes));
        let uppercase = format!("0x{}", hex::encode_upper(bytes));

        let address_mixed = Address::new(&mixed).unwrap();
        let address_lower = Address::new(&lowercase).unwrap();
        let address_upper = Address::new(&uppercase).unwrap();

        assert_eq!(
            address_mixed, address_lower,
            "PartialEq must treat mixed-case and lowercase variants as equal"
        );
        assert_eq!(
            address_upper, address_lower,
            "PartialEq must treat uppercase and lowercase variants as equal"
        );
        assert_eq!(
            address_mixed.as_str(),
            mixed,
            "as_str must preserve the original input casing"
        );

        let mut map = HashMap::new();
        map.insert(address_mixed.clone(), "value");
        assert_eq!(
            map.get(&address_lower),
            Some(&"value"),
            "hash must agree with PartialEq for lowercase lookup"
        );
        assert_eq!(
            map.get(&address_upper),
            Some(&"value"),
            "hash must agree with PartialEq for uppercase lookup"
        );

        let mut set = HashSet::new();
        set.insert(address_mixed.clone());
        set.insert(address_lower.clone());
        set.insert(address_upper.clone());
        assert_eq!(
            set.len(),
            1,
            "a HashSet must collapse case-variant addresses into one element"
        );
    }
}

#[test]
fn address_rejects_malformed_inputs() {
    assert!(
        Address::new("").is_err(),
        "empty string must not parse as an address"
    );
    assert!(
        Address::new("742d35cc6634c0532925a3b844bc9e7595f0bebd").is_err(),
        "missing 0x prefix must fail closed"
    );
    assert!(
        Address::new("0X742d35cc6634c0532925a3b844bc9e7595f0bebd").is_err(),
        "uppercase 0X prefix must fail closed"
    );

    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed);
        let bytes = rng.fill::<20>();
        let canonical = format!("0x{}", hex::encode(bytes));

        let too_short = format!("0x{}", &hex::encode(bytes)[..38]);
        assert!(
            Address::new(&too_short).is_err(),
            "short address input {too_short} must fail closed"
        );

        let mut too_long = canonical.clone();
        too_long.push_str("00");
        assert!(
            Address::new(&too_long).is_err(),
            "long address input {too_long} must fail closed"
        );

        let mut nonhex = canonical.as_bytes().to_vec();
        let flip = (rng.next_u32() as usize) % 20 + 2;
        nonhex[flip] = b'g';
        let nonhex = String::from_utf8(nonhex).unwrap();
        assert!(
            Address::new(&nonhex).is_err(),
            "non-hex character in {nonhex} must fail closed"
        );
    }
}

#[test]
fn amount_canonical_decimal_matches_hex_equivalent() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed);
        let (value, canonical, hex_form) = rng.decimal_amount_components();

        let from_decimal = Amount::new(&canonical).unwrap();
        let from_hex = Amount::new(&hex_form).unwrap();

        assert_eq!(
            from_decimal, from_hex,
            "decimal and hex inputs representing the same value must compare equal"
        );
        assert_eq!(
            from_decimal.as_str(),
            canonical,
            "canonical form must be the base-10 representation of the input"
        );

        let roundtrip = Amount::new(from_decimal.as_str()).unwrap();
        assert_eq!(
            roundtrip, from_decimal,
            "feeding the canonical form back into Amount::new must roundtrip"
        );

        let reparsed = BigUint::parse_bytes(from_decimal.as_str().as_bytes(), 10).unwrap();
        assert_eq!(
            reparsed, value,
            "canonical Amount string must parse back to the original BigUint"
        );
    }

    assert_eq!(
        Amount::new("0").unwrap(),
        Amount::zero(),
        "Amount::zero must match the canonical decimal zero"
    );
    assert_eq!(
        Amount::new("0x0").unwrap(),
        Amount::zero(),
        "Amount::zero must match the canonical hex zero"
    );
    assert_eq!(
        Amount::new("0x00000000").unwrap(),
        Amount::zero(),
        "leading hex zeros must normalize to canonical zero"
    );
}

#[test]
fn amount_rejects_malformed_and_out_of_range_inputs() {
    assert!(
        Amount::new("").is_err(),
        "empty input must not parse as an amount"
    );
    assert!(
        Amount::new("-1").is_err(),
        "negative decimal inputs must fail closed"
    );
    assert!(
        Amount::new("0xg").is_err(),
        "non-hex characters after 0x prefix must fail closed"
    );
    assert!(
        Amount::new("1.5").is_err(),
        "non-integer decimals must fail closed"
    );

    let overflow_hex = format!("0x1{}", "0".repeat(64));
    assert!(
        Amount::new(&overflow_hex).is_err(),
        "inputs beyond 256 bits must fail closed"
    );

    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed);
        let bytes = rng.fill::<4>();
        let garbled = format!("0x{}gg", hex::encode(bytes));
        assert!(
            Amount::new(&garbled).is_err(),
            "hex input {garbled} with non-hex suffix must fail closed"
        );
    }
}

#[test]
fn hash32_family_roundtrip_preserves_input() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed);
        let bytes = rng.fill::<32>();
        let canonical = format!("0x{}", hex::encode(bytes));
        let mixed = mixed_case_hex(&mut rng, &bytes);

        let hash = Hash32::new(&canonical).unwrap();
        assert_eq!(
            hash.as_str(),
            canonical,
            "Hash32 must preserve input string"
        );

        let hash_mixed = Hash32::new(&mixed).unwrap();
        assert_eq!(
            hash_mixed.as_str(),
            mixed,
            "Hash32 must preserve the exact input case"
        );

        let rebuilt = Hash32::new(hash.as_str()).unwrap();
        assert_eq!(
            rebuilt, hash,
            "Hash32 must roundtrip through its string form"
        );
    }
}

#[test]
fn hash32_rejects_malformed_inputs() {
    assert!(Hash32::new("").is_err());
    assert!(Hash32::new("0x").is_err(), "empty payload must fail closed");

    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed);
        let canonical = format!("0x{}", hex::encode(rng.fill::<32>()));

        let too_short = format!("0x{}", &canonical[2..canonical.len() - 2]);
        assert!(
            Hash32::new(&too_short).is_err(),
            "short hex {too_short} must fail closed"
        );

        let too_long = format!("{canonical}00");
        assert!(
            Hash32::new(&too_long).is_err(),
            "long hex {too_long} must fail closed"
        );

        let mut nonhex = canonical.as_bytes().to_vec();
        let flip = (rng.next_u32() as usize) % 64 + 2;
        nonhex[flip] = b'z';
        let nonhex = String::from_utf8(nonhex).unwrap();
        assert!(
            Hash32::new(&nonhex).is_err(),
            "non-hex character in {nonhex} must fail closed"
        );
    }
}

#[test]
fn app_data_hex_roundtrip_preserves_input_and_rejects_malformed_inputs() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed);
        let bytes = rng.fill::<32>();
        let canonical = format!("0x{}", hex::encode(bytes));

        let app_data = AppDataHex::new(&canonical).unwrap();
        assert_eq!(app_data.as_str(), canonical);

        let rebuilt = AppDataHex::new(app_data.as_str()).unwrap();
        assert_eq!(
            rebuilt, app_data,
            "AppDataHex must roundtrip through its own string form"
        );

        let too_short = format!("0x{}", &hex::encode(bytes)[..62]);
        assert!(
            AppDataHex::new(&too_short).is_err(),
            "short AppDataHex {too_short} must fail closed"
        );

        let too_long = format!("{canonical}00");
        assert!(
            AppDataHex::new(&too_long).is_err(),
            "long AppDataHex {too_long} must fail closed"
        );

        let missing_prefix = hex::encode(bytes);
        assert!(
            AppDataHex::new(&missing_prefix).is_err(),
            "AppDataHex without 0x prefix must fail closed"
        );
    }
}

#[test]
fn order_uid_roundtrip_preserves_input_and_rejects_malformed_inputs() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed);
        let bytes = rng.fill::<56>();
        let canonical = format!("0x{}", hex::encode(bytes));

        let uid = OrderUid::new(&canonical).unwrap();
        assert_eq!(uid.as_str(), canonical);

        let rebuilt = OrderUid::new(uid.as_str()).unwrap();
        assert_eq!(
            rebuilt, uid,
            "OrderUid must roundtrip through its own string form"
        );

        let too_short = format!("0x{}", &hex::encode(bytes)[..110]);
        assert!(
            OrderUid::new(&too_short).is_err(),
            "short OrderUid {too_short} must fail closed"
        );

        let too_long = format!("{canonical}00");
        assert!(
            OrderUid::new(&too_long).is_err(),
            "long OrderUid {too_long} must fail closed"
        );
    }
}

#[test]
fn hex_data_accepts_empty_payload_and_preserves_valid_inputs() {
    let empty = HexData::empty();
    assert_eq!(
        empty.as_str(),
        "0x",
        "canonical empty payload is literally 0x"
    );

    let default: HexData = HexData::default();
    assert_eq!(
        default, empty,
        "default HexData must match HexData::empty()"
    );

    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed);
        let bytes = rng.fill::<32>();
        let canonical = format!("0x{}", hex::encode(bytes));

        let data = HexData::new(&canonical).unwrap();
        assert_eq!(
            data.as_str(),
            canonical,
            "HexData must preserve valid inputs byte-for-byte"
        );

        let rebuilt = HexData::new(data.as_str()).unwrap();
        assert_eq!(rebuilt, data);
    }
}

#[test]
fn token_id_is_chain_and_address_sensitive() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed);
        let address_a = Address::new(format!("0x{}", hex::encode(rng.fill::<20>()))).unwrap();
        let address_b = Address::new(format!("0x{}", hex::encode(rng.fill::<20>()))).unwrap();

        let chain_a: ChainId = rng.supported_chain().into();
        let mut chain_b: ChainId = rng.supported_chain().into();
        while chain_b == chain_a {
            chain_b = rng.supported_chain().into();
        }

        let id_same = token_id(chain_a, &address_a);
        let id_same_again = token_id(chain_a, &address_a);
        assert_eq!(
            id_same, id_same_again,
            "token_id must be deterministic for identical inputs"
        );

        let id_different_address = token_id(chain_a, &address_b);
        assert_ne!(
            id_same, id_different_address,
            "token_id must change when the address changes"
        );

        let id_different_chain = token_id(chain_b, &address_a);
        assert_ne!(
            id_same, id_different_chain,
            "token_id must change when the chain changes"
        );
    }
}

#[test]
fn supported_chain_id_roundtrips_through_chain_id() {
    for supported in SupportedChainId::ALL {
        let raw: ChainId = supported.into();
        let rebuilt = SupportedChainId::try_from(raw)
            .expect("every supported chain id must roundtrip through its numeric form");
        assert_eq!(
            supported, rebuilt,
            "numeric roundtrip must preserve the supported chain value"
        );
    }

    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed);
        let mut candidate = rng.next_u64();
        while SupportedChainId::ALL
            .iter()
            .any(|chain| ChainId::from(*chain) == candidate)
        {
            candidate = candidate.wrapping_add(1);
        }
        assert!(
            SupportedChainId::try_from(candidate).is_err(),
            "unsupported chain id {candidate} must fail closed"
        );
    }
}
