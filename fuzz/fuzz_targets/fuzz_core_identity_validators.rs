#![no_main]

//! Fuzz target for the typed core identity validators.
//!
//! **Surface:** `cow_sdk_core::{Address, HexData, AppDataHash, Hash32,
//! OrderUid}::new`.
//! **Property:** `PROP-CORE-003`.
//! **Seed contract:** corpus inputs cover the canonical zero-form of every
//! identity type plus boundary lengths and adversarial mixed-case / wrong
//! prefix / non-hex inputs.
//! **Corpus README:** `../corpus/fuzz_core_identity_validators/README.md`.
//!
//! The first byte of the input picks one of the five typed constructors and
//! the remaining bytes pass through `String::from_utf8_lossy` into the
//! constructor. Successful constructions must round-trip through their
//! `Display` form and exercise the documented hex-character length
//! constraint. Failures are typed errors; no panic is allowed.

use cow_sdk_core::{Address, AppDataHash, Hash32, HexData, OrderUid};
use libfuzzer_sys::fuzz_target;

const ADDRESS_LEN: usize = 42;
const HASH32_LEN: usize = 66;
const APP_DATA_HASH_LEN: usize = 66;
const ORDER_UID_LEN: usize = 114;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let (discriminant, rest) = (data[0], &data[1..]);
    let raw = String::from_utf8_lossy(rest).into_owned();

    match discriminant % 5 {
        0 => exercise_address(&raw),
        1 => exercise_hash32(&raw),
        2 => exercise_app_data_hash(&raw),
        3 => exercise_order_uid(&raw),
        _ => exercise_hex_data(&raw),
    }
});

fn exercise_address(raw: &str) {
    let first = Address::new(raw.to_owned());
    let second = Address::new(raw.to_owned());
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "Address::new must be deterministic on identical input",
    );
    if let (Some(left), Some(right)) = (first.as_ref().ok(), second.as_ref().ok()) {
        assert_eq!(
            left, right,
            "Address::new must produce identical typed values for identical input",
        );
    }
    if let Ok(address) = first {
        let rendered = address.to_string();
        assert_eq!(
            rendered.len(),
            ADDRESS_LEN,
            "Address Display must remain the canonical 42-char `0x`-prefixed hex form",
        );
        let roundtrip = Address::new(rendered.clone())
            .expect("Display form of an accepted Address must re-parse");
        assert_eq!(
            address, roundtrip,
            "Address::new round-trip through Display form must be stable",
        );
    }
}

fn exercise_hash32(raw: &str) {
    let first = Hash32::new(raw.to_owned());
    let second = Hash32::new(raw.to_owned());
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "Hash32::new must be deterministic on identical input",
    );
    if let (Some(left), Some(right)) = (first.as_ref().ok(), second.as_ref().ok()) {
        assert_eq!(
            left, right,
            "Hash32::new must produce identical typed values for identical input",
        );
    }
    if let Ok(value) = first {
        let rendered = value.to_string();
        assert_eq!(
            rendered.len(),
            HASH32_LEN,
            "Hash32 Display must remain the canonical 66-char `0x`-prefixed hex form",
        );
        let roundtrip = Hash32::new(rendered)
            .expect("Display form of an accepted Hash32 must re-parse");
        assert_eq!(
            value, roundtrip,
            "Hash32::new round-trip through Display form must be stable",
        );
    }
}

fn exercise_app_data_hash(raw: &str) {
    let first = AppDataHash::new(raw.to_owned());
    let second = AppDataHash::new(raw.to_owned());
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "AppDataHash::new must be deterministic on identical input",
    );
    if let (Some(left), Some(right)) = (first.as_ref().ok(), second.as_ref().ok()) {
        assert_eq!(
            left, right,
            "AppDataHash::new must produce identical typed values for identical input",
        );
    }
    if let Ok(value) = first {
        let rendered = value.to_string();
        assert_eq!(
            rendered.len(),
            APP_DATA_HASH_LEN,
            "AppDataHash Display must remain the canonical 66-char `0x`-prefixed hex form",
        );
        let roundtrip = AppDataHash::new(rendered)
            .expect("Display form of an accepted AppDataHash must re-parse");
        assert_eq!(
            value, roundtrip,
            "AppDataHash::new round-trip through Display form must be stable",
        );
    }
}

fn exercise_order_uid(raw: &str) {
    let first = OrderUid::new(raw.to_owned());
    let second = OrderUid::new(raw.to_owned());
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "OrderUid::new must be deterministic on identical input",
    );
    if let (Some(left), Some(right)) = (first.as_ref().ok(), second.as_ref().ok()) {
        assert_eq!(
            left, right,
            "OrderUid::new must produce identical typed values for identical input",
        );
    }
    if let Ok(value) = first {
        let rendered = value.to_string();
        assert_eq!(
            rendered.len(),
            ORDER_UID_LEN,
            "OrderUid Display must remain the canonical 114-char `0x`-prefixed hex form",
        );
        let roundtrip = OrderUid::new(rendered)
            .expect("Display form of an accepted OrderUid must re-parse");
        assert_eq!(
            value, roundtrip,
            "OrderUid::new round-trip through Display form must be stable",
        );
    }
}

fn exercise_hex_data(raw: &str) {
    let first = HexData::new(raw.to_owned());
    let second = HexData::new(raw.to_owned());
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "HexData::new must be deterministic on identical input",
    );
    if let (Some(left), Some(right)) = (first.as_ref().ok(), second.as_ref().ok()) {
        assert_eq!(
            left, right,
            "HexData::new must produce identical typed values for identical input",
        );
    }
    if let Ok(value) = first {
        let rendered = value.to_string();
        assert!(
            rendered.starts_with("0x"),
            "HexData Display must always carry the `0x` prefix",
        );
        let payload_len = rendered.len() - 2;
        assert!(
            payload_len.is_multiple_of(2),
            "HexData stored payload must have an even hex-character count (odd-length \
             inputs are normalized with a leading zero nibble): rendered = {rendered}",
        );
        let roundtrip = HexData::new(rendered)
            .expect("Display form of an accepted HexData must re-parse");
        assert_eq!(
            value, roundtrip,
            "HexData::new round-trip through Display form must be stable",
        );
    }
}
