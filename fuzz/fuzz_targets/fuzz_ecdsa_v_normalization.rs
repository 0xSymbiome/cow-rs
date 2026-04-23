#![no_main]

//! Fuzz target for canonical ECDSA `v`-byte normalization.
//!
//! Drives arbitrary 65-byte signatures through
//! [`cow_sdk_contracts::normalized_ecdsa_signature`] and asserts the
//! following invariants:
//!
//! * every accepted output carries `v ∈ {27, 28}`
//! * bytes `0..64` (`r || s`) are preserved byte-identically
//! * `{0, 27}` map to `27` and `{1, 28}` map to `28`
//! * every rejected input fails specifically through
//!   [`ContractsError::InvalidSignatureRecoveryByte`]

use cow_sdk_contracts::{ContractsError, normalized_ecdsa_signature};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: [u8; 65]| {
    let input = format!("0x{}", hex::encode(data));

    match normalized_ecdsa_signature(&input) {
        Ok(normalized) => {
            let output = hex::decode(normalized.trim_start_matches("0x"))
                .expect("normalized signatures must remain valid hex");
            assert_eq!(
                output.len(),
                65,
                "accepted signatures must stay 65 bytes after normalization",
            );
            assert_eq!(
                &output[..64],
                &data[..64],
                "normalization must preserve the r||s bytes byte-identically",
            );

            let input_v = data[64];
            let output_v = output[64];
            assert!(
                matches!(output_v, 27 | 28),
                "normalized v must be 27 or 28, got {output_v}",
            );
            assert!(
                (matches!(input_v, 0 | 27) && output_v == 27)
                    || (matches!(input_v, 1 | 28) && output_v == 28),
                "input v={input_v} mapped incorrectly to output v={output_v}",
            );
        }
        Err(ContractsError::InvalidSignatureRecoveryByte { value }) => {
            assert_eq!(
                value, data[64],
                "rejected v byte must be surfaced unchanged in the typed error",
            );
            assert!(
                !matches!(data[64], 0 | 1 | 27 | 28),
                "accepted recovery bytes must not reject, but v={} did",
                data[64],
            );
        }
        Err(other) => panic!("unexpected error class for valid 65-byte hex input: {other:?}"),
    }
});
