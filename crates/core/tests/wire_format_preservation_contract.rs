//! Wire-form preservation contract for the cow identity primitives.
//!
//! Each test pins the canonical lowercase `0x`-prefixed hexadecimal
//! string that the protocol's TypeScript SDK and the
//! `parity/fixtures/**/*.json` corpora exchange on the wire. These
//! contracts must stay true today (against the cow string newtypes) and
//! after the upcoming staged collapse onto `alloy_primitives` per
//! ADR 0052. Every test exercises both the direct accessor surface and
//! the JSON serde round-trip so the wire bytes stay locked through
//! either path.

use cow_sdk_core::prelude::{AddressExt, Hash32Ext, HexDataExt, OrderUidExt};
use cow_sdk_core::{Address, AppDataHash, Hash32, HexData, OrderUid};

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
    assert_eq!(address.as_str(), ADDRESS_HEX);

    let json = serde_json::to_string(&address).expect("Address must serialize");
    assert_eq!(json, format!("\"{ADDRESS_HEX}\""));

    let round_trip: Address =
        serde_json::from_str(&json).expect("Address must deserialize round trip");
    assert_eq!(round_trip.as_str(), ADDRESS_HEX);
}

#[test]
fn address_zero_round_trips_with_canonical_zero_bytes() {
    let zero = Address::new(ADDRESS_ZERO_HEX).expect("zero address must parse");
    assert_eq!(zero.as_str(), ADDRESS_ZERO_HEX);
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
    assert_eq!(hash.as_str(), HASH32_HEX);

    let json = serde_json::to_string(&hash).expect("Hash32 must serialize");
    assert_eq!(json, format!("\"{HASH32_HEX}\""));

    let round_trip: Hash32 =
        serde_json::from_str(&json).expect("Hash32 must deserialize round trip");
    assert_eq!(round_trip.as_str(), HASH32_HEX);
}

#[test]
fn hash32_zero_round_trips_with_canonical_zero_bytes() {
    let zero = Hash32::new(HASH32_ZERO_HEX).expect("zero Hash32 must parse");
    assert_eq!(zero.as_str(), HASH32_ZERO_HEX);
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

// ---- AppDataHash (Hash32-shaped) --------------------------------------

#[test]
fn app_data_hash_wire_form_matches_canonical_protocol_constant() {
    let hash = AppDataHash::new(APP_DATA_HEX).expect("canonical AppDataHash must parse");
    assert_eq!(hash.as_str(), APP_DATA_HEX);

    let json = serde_json::to_string(&hash).expect("AppDataHash must serialize");
    assert_eq!(json, format!("\"{APP_DATA_HEX}\""));

    let round_trip: AppDataHash =
        serde_json::from_str(&json).expect("AppDataHash must deserialize round trip");
    assert_eq!(round_trip.as_str(), APP_DATA_HEX);
}

// ---- HexData (variable length) ----------------------------------------

#[test]
fn hex_data_wire_form_preserves_0x_prefix_across_variable_length() {
    let payload = HexData::new(HEX_DATA_FOUR_BYTES).expect("canonical HexData must parse");
    assert_eq!(payload.as_str(), HEX_DATA_FOUR_BYTES);

    let json = serde_json::to_string(&payload).expect("HexData must serialize");
    assert_eq!(json, format!("\"{HEX_DATA_FOUR_BYTES}\""));

    let round_trip: HexData =
        serde_json::from_str(&json).expect("HexData must deserialize round trip");
    assert_eq!(round_trip.as_str(), HEX_DATA_FOUR_BYTES);
}

#[test]
fn hex_data_empty_round_trips_with_bare_0x_prefix() {
    let empty = HexData::new(HEX_DATA_EMPTY).expect("empty HexData must parse");
    assert_eq!(empty.as_str(), HEX_DATA_EMPTY);

    let json = serde_json::to_string(&empty).expect("empty HexData must serialize");
    assert_eq!(json, format!("\"{HEX_DATA_EMPTY}\""));
}

#[test]
fn hex_data_pads_odd_length_input_with_leading_zero_nibble() {
    let padded = HexData::new("0x123").expect("odd-length HexData must left-pad");
    assert_eq!(padded.as_str(), "0x0123");
    assert_eq!(padded.byte_length(), 2);
}

// ---- OrderUid (56 bytes fixed) ----------------------------------------

#[test]
fn order_uid_wire_form_is_lowercase_0x_prefixed_114_chars() {
    let uid = OrderUid::new(ORDER_UID_HEX).expect("canonical OrderUid must parse");
    assert_eq!(uid.as_str(), ORDER_UID_HEX);

    let json = serde_json::to_string(&uid).expect("OrderUid must serialize");
    assert_eq!(json, format!("\"{ORDER_UID_HEX}\""));

    let round_trip: OrderUid =
        serde_json::from_str(&json).expect("OrderUid must deserialize round trip");
    assert_eq!(round_trip.as_str(), ORDER_UID_HEX);
}

#[test]
fn order_uid_rejects_malformed_inputs() {
    assert!(OrderUid::new("").is_err());
    assert!(OrderUid::new("not-hex").is_err());
    assert!(OrderUid::new("0x1234").is_err());
    assert!(serde_json::from_str::<OrderUid>(r#""0x""#).is_err());
}

// ---- Extension trait surface mirrors the inherent surface -------------

#[test]
fn identity_ext_traits_mirror_cow_inherent_accessors() {
    use alloy_primitives::{Address as AlloyAddress, B256, Bytes, FixedBytes};

    let alloy_address = <AlloyAddress as AddressExt>::new(ADDRESS_HEX).unwrap();
    let cow_address = Address::new(ADDRESS_HEX).unwrap();
    assert_eq!(AddressExt::as_str(&alloy_address), cow_address.as_str());

    let alloy_hash = <B256 as Hash32Ext>::new(HASH32_HEX).unwrap();
    let cow_hash = Hash32::new(HASH32_HEX).unwrap();
    assert_eq!(Hash32Ext::as_str(&alloy_hash), cow_hash.as_str());

    let alloy_bytes = <Bytes as HexDataExt>::new(HEX_DATA_FOUR_BYTES).unwrap();
    let cow_bytes = HexData::new(HEX_DATA_FOUR_BYTES).unwrap();
    assert_eq!(HexDataExt::as_str(&alloy_bytes), cow_bytes.as_str());

    let alloy_uid = <FixedBytes<56> as OrderUidExt>::new(ORDER_UID_HEX).unwrap();
    let cow_uid = OrderUid::new(ORDER_UID_HEX).unwrap();
    assert_eq!(OrderUidExt::as_str(&alloy_uid), cow_uid.as_str());
}
