#![no_main]

//! Fuzz target for the EIP-1271 verifier-payload codec round-trip.
//!
//! **Surface:** `cow_sdk_contracts::encode_eip1271_signature_data`
//! paired with `cow_sdk_contracts::decode_eip1271_signature_data`.
//! **Property:** `PROP-CON-005`.
//!
//! Constructs an `Eip1271SignatureData` from a 20-byte verifier and an
//! arbitrary-length signature payload (capped at 256 bytes), then asserts
//! that the encode/decode round-trip preserves the value byte-for-byte
//! and that the encoder is deterministic across `encode/decode/encode`.

use cow_sdk_contracts::{
    Eip1271SignatureData, decode_eip1271_signature_data, encode_eip1271_signature_data,
};
use cow_sdk_core::{Address, HexData};
use libfuzzer_sys::fuzz_target;

const MAX_EIP1271_SIGNATURE_BYTES: usize = 256;

fuzz_target!(|data: &[u8]| {
    let (verifier_bytes, sig_bytes) = match data.split_at_checked(20) {
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
        HexData::from_bytes(payload.to_vec()),
    );

    let encoded = encode_eip1271_signature_data(&data)
        .expect("encode_eip1271_signature_data must accept well-formed input");
    let decoded = decode_eip1271_signature_data(&encoded)
        .expect("decode_eip1271_signature_data must accept its own encoder output");

    assert_eq!(
        data, decoded,
        "decode(encode(data)) must equal the original Eip1271SignatureData",
    );

    let encoded_again = encode_eip1271_signature_data(&decoded)
        .expect("re-encoding the decoded value must succeed");
    assert_eq!(
        encoded, encoded_again,
        "EIP-1271 encoding must be deterministic across encode/decode/encode",
    );
});
