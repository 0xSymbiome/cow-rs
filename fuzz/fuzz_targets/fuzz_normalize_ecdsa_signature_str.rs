#![no_main]

//! Fuzz target for the string-typed ECDSA normalization and EIP-1271
//! signature-data codec surfaces.
//!
//! **Surface:** `cow_sdk_contracts::normalized_ecdsa_signature(&str)`
//! plus the round-trip pair
//! `cow_sdk_contracts::encode_eip1271_signature_data` /
//! `cow_sdk_contracts::decode_eip1271_signature_data`.
//! **Property:** `PROP-CON-005`.
//! **Seed contract:** corpus inputs cover canonical 65-byte hex
//! signatures with each accepted `v` byte (`0`, `1`, `27`, `28`),
//! boundary inputs (empty string, short hex, missing prefix, all-`0xff`
//! payload), an adversarial mixed-case-`v` mutation, and a multi-shape
//! EIP-1271 round-trip seed that toggles the discriminant byte to the
//! odd branch.
//! **Corpus README:** `../corpus/fuzz_normalize_ecdsa_signature_str/README.md`.
//!
//! The first byte selects the branch: even discriminants drive the
//! string-typed normalizer with the remaining bytes converted to a UTF-8
//! string via `String::from_utf8_lossy`; odd discriminants build an
//! `Eip1271SignatureData` instance from the trailing bytes and assert
//! `decode(encode(data)) == data`.
//!
//! Invariants:
//!
//! * `normalized_ecdsa_signature(&str)` never panics for any input.
//! * Accepted outputs are 65 bytes, lowercase, `0x`-prefixed hex, and
//!   carry `v ∈ {27, 28}`.
//! * `normalized_ecdsa_signature` is idempotent on accepted outputs.
//! * `decode_eip1271_signature_data(encode_eip1271_signature_data(data))
//!   == data` for every constructible `Eip1271SignatureData`.

use cow_sdk_contracts::{
    Eip1271SignatureData, decode_eip1271_signature_data, encode_eip1271_signature_data,
    normalized_ecdsa_signature,
};
use cow_sdk_core::Address;
use libfuzzer_sys::fuzz_target;

const MAX_EIP1271_SIGNATURE_BYTES: usize = 256;

fuzz_target!(|data: &[u8]| {
    let Some((discriminant, rest)) = data.split_first() else {
        // No bytes available; defensively drive the normalizer with an
        // empty string so the empty-input panic class is also covered.
        let _ = normalized_ecdsa_signature("");
        return;
    };

    if *discriminant % 2 == 0 {
        exercise_normalize(rest);
    } else {
        exercise_eip1271_roundtrip(rest);
    }
});

fn exercise_normalize(rest: &[u8]) {
    let candidate = String::from_utf8_lossy(rest).into_owned();

    let Ok(first) = normalized_ecdsa_signature(&candidate) else {
        return;
    };

    assert!(
        first.starts_with("0x"),
        "accepted normalized signature must keep the 0x prefix, got {first:?}",
    );
    let stripped = &first[2..];
    assert_eq!(
        stripped.len(),
        130,
        "accepted signature hex body must be 130 ASCII chars (65 bytes), got {} chars",
        stripped.len(),
    );
    assert!(
        stripped
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "accepted normalized signature must be lowercase hex, got {first:?}",
    );

    let bytes = hex::decode(stripped)
        .expect("normalized_ecdsa_signature accepted output must decode as valid hex");
    assert_eq!(
        bytes.len(),
        65,
        "decoded signature must be exactly 65 bytes",
    );
    assert!(
        matches!(bytes[64], 27 | 28),
        "normalized signature v byte must be 27 or 28, got {}",
        bytes[64],
    );

    // Idempotency: feeding an accepted output back through the
    // normalizer must produce the same canonical string.
    let second = normalized_ecdsa_signature(&first)
        .expect("normalized output must round-trip through the normalizer");
    assert_eq!(
        first, second,
        "normalized_ecdsa_signature must be idempotent on accepted outputs",
    );
}

fn exercise_eip1271_roundtrip(rest: &[u8]) {
    let (verifier_bytes, sig_bytes) = match rest.split_at_checked(20) {
        Some((head, tail)) => {
            let mut head_array = [0u8; 20];
            head_array.copy_from_slice(head);
            (head_array, tail)
        }
        None => return,
    };

    let payload_len = sig_bytes.len().min(MAX_EIP1271_SIGNATURE_BYTES);
    let payload = &sig_bytes[..payload_len];

    let data = Eip1271SignatureData::new(
        Address::from_bytes(verifier_bytes),
        format!("0x{}", hex::encode(payload)),
    );

    let encoded = encode_eip1271_signature_data(&data)
        .expect("encode_eip1271_signature_data must accept well-formed input");
    let decoded = decode_eip1271_signature_data(&encoded)
        .expect("decode_eip1271_signature_data must accept its own encoder output");

    assert_eq!(
        data, decoded,
        "decode(encode(data)) must equal the original Eip1271SignatureData",
    );

    // Determinism: a second encode pass on the decoded value must
    // produce the byte-identical hex form.
    let encoded_again = encode_eip1271_signature_data(&decoded)
        .expect("re-encoding the decoded value must succeed");
    assert_eq!(
        encoded, encoded_again,
        "EIP-1271 encoding must be deterministic across encode/decode/encode",
    );
}
