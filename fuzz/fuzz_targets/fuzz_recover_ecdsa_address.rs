#![no_main]

//! Fuzz target for `Signature::recover_ecdsa_address`.
//!
//! **Property:** `PROP-CON-012`.
//!
//! Drives arbitrary 32-byte digests, 65-byte ECDSA signatures, and a
//! scheme-selector byte through `Signature::recover_ecdsa_address` and
//! asserts:
//!
//! * Every accepted output is a 20-byte `0x`-prefixed address.
//! * Every rejection falls into the typed error partition
//!   (`InvalidSignatureRecoveryByte`, `SignatureRecovery`,
//!   `InvalidSignatureLength`, `DecodeHex`, `InvalidHexPrefix`,
//!   `InvalidDecodedLength`) rather than panicking or returning a
//!   broader untyped error.
//!
//! The structured-input width is bounded by the `Arbitrary` derive on
//! the `Input` struct (98 bytes per run).

use arbitrary::Arbitrary;
use cow_sdk_contracts::{ContractsError, Signature, SigningScheme};
use cow_sdk_core::Hash32;
use libfuzzer_sys::fuzz_target;

#[derive(Debug, Arbitrary)]
struct Input {
    digest: [u8; 32],
    signature: [u8; 65],
    scheme: u8,
}

fuzz_target!(|input: Input| {
    let digest = Hash32::new(format!("0x{}", hex::encode(input.digest)))
        .expect("fixed-width digest bytes must form a valid Hash32");
    let scheme = if input.scheme % 2 == 0 {
        SigningScheme::Eip712
    } else {
        SigningScheme::EthSign
    };
    let signature = Signature::Ecdsa {
        scheme,
        data: format!("0x{}", hex::encode(input.signature)),
    };

    let first = signature.recover_ecdsa_address(&digest);
    let second = signature.recover_ecdsa_address(&digest);
    match (&first, &second) {
        (Ok(left), Ok(right)) => {
            let left_hex = left.to_hex_string();
            let right_hex = right.to_hex_string();
            assert_eq!(
                left_hex, right_hex,
                "recover_ecdsa_address must be deterministic for identical input",
            );
            assert_eq!(left.as_slice().len(), 20, "recovered address must be 20 bytes");
            assert!(
                left_hex.starts_with("0x"),
                "recovered address must be 0x-prefixed",
            );
            assert_eq!(
                left_hex.len(),
                42,
                "recovered address must be exactly 42 characters (0x + 40 hex)",
            );
            assert!(
                left_hex[2..].chars().all(|c: char| c.is_ascii_hexdigit()),
                "recovered address tail must be ASCII hex only",
            );
        }
        (Err(_), Err(_)) => {}
        _ => panic!(
            "recover_ecdsa_address must be deterministic; got first={first:?} second={second:?}",
        ),
    }
    match first {
        Ok(_) => {}
        Err(
            ContractsError::InvalidSignatureRecoveryByte { .. }
            | ContractsError::SignatureRecovery { .. }
            | ContractsError::InvalidSignatureLength { .. }
            | ContractsError::DecodeHex { .. }
            | ContractsError::InvalidHexPrefix { .. }
            | ContractsError::InvalidDecodedLength { .. },
        ) => {}
        Err(error) => panic!("unexpected recovery error: {error:?}"),
    }
});
