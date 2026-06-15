#![no_main]

//! Fuzz target for canonical ECDSA `v`-byte normalization.
//!
//! **Property:** `PROP-CON-005`.
//! Drives arbitrary 65-byte signatures through
//! [`cow_sdk_contracts::RecoverableSignature::parse_bytes`] and asserts
//! the following invariants:
//!
//! * every accepted canonical output carries `v ∈ {27, 28}`
//! * bytes `0..64` (`r || s`) are preserved byte-identically
//! * `{0, 27}` map to `27` and `{1, 28}` map to `28`
//! * every rejected input fails specifically through
//!   [`ContractsError::InvalidSignatureRecoveryByte`]

use cow_sdk_contracts::{ContractsError, RecoverableSignature};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: [u8; 65]| {
    match RecoverableSignature::parse_bytes(&data) {
        Ok(sig) => {
            let output = sig.to_bytes();
            assert_eq!(
                &output[..64],
                &data[..64],
                "canonicalisation must preserve the r||s bytes byte-identically",
            );

            let input_v = data[64];
            let output_v = output[64];
            assert!(
                matches!(output_v, 27 | 28),
                "canonical v must be 27 or 28, got {output_v}",
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
        Err(other) => panic!("unexpected error class for valid 65-byte input: {other:?}"),
    }
});
