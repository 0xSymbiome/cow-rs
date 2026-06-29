#![no_main]

//! Differential fuzz target: cow `RecoverableSignature::parse_bytes`
//! rejection set is a strict refinement of alloy
//! [`alloy_primitives::Signature::from_raw`].
//!
//! **Property:** `PROP-CON-005`.
//!
//! Drives arbitrary 65-byte inputs through both surfaces and asserts:
//!
//! * If the cow surface accepts the payload, alloy must also accept it
//!   (the cow accept set is a subset of alloy's).
//! * If the cow surface rejects through
//!   [`ContractsError::InvalidSignatureRecoveryByte`], either alloy
//!   accepted it (the proper-subset case — the strict-narrowing payoff)
//!   or alloy rejected it through [`alloy_primitives::SignatureError::InvalidParity`]
//!   with the same trailing byte (the agreed-rejection case).
//! * When both surfaces accept, the canonical 65-byte output of
//!   [`RecoverableSignature::to_bytes`] matches alloy's
//!   [`alloy_primitives::Signature::as_bytes`] byte-for-byte.
//! * The cow accept set is exactly `{0, 1, 27, 28}`; the cow rejection
//!   set covers every other byte. The differential fuzz proves alloy
//!   has a strictly wider accept window without re-encoding alloy's
//!   `normalize_v` logic in this corpus.

use alloy_primitives::Signature as AlloySignature;
use cow_sdk_contracts::{ContractsError, RecoverableSignature};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: [u8; 65]| {
    let cow = RecoverableSignature::parse_bytes(&data);
    let alloy = AlloySignature::from_raw(&data);

    match (cow, alloy) {
        (Ok(cow_sig), Ok(alloy_sig)) => {
            assert!(
                matches!(data[64], 0 | 1 | 27 | 28),
                "cow accepted v = {}, which is outside the ADR 0022 accept set",
                data[64],
            );
            assert_eq!(
                cow_sig.to_bytes(),
                alloy_sig.as_bytes(),
                "cow and alloy must produce byte-identical canonical bytes when both accept",
            );
            let canonical_v = cow_sig.to_bytes()[64];
            assert!(
                matches!(canonical_v, 27 | 28),
                "canonical v must be in {{27, 28}}, got {canonical_v}",
            );
        }
        (Err(ContractsError::InvalidSignatureRecoveryByte { value }), Ok(_)) => {
            assert_eq!(
                value, data[64],
                "cow rejection v-byte must match the input trailing byte",
            );
            assert!(
                !matches!(data[64], 0 | 1 | 27 | 28),
                "cow must accept v in {{0, 1, 27, 28}}, but rejected {}",
                data[64],
            );
            // alloy accepted because data[64] is in alloy's wider accept set
            // (EIP-155 v >= 35). This is the proper-subset payoff of the
            // strict cow contract.
        }
        (Err(ContractsError::InvalidSignatureRecoveryByte { value }), Err(alloy_err)) => {
            assert_eq!(value, data[64]);
            // Agreed rejection: alloy also rejects on parity. The
            // alloy-side reason must be InvalidParity for the same v.
            match alloy_err {
                alloy_primitives::SignatureError::InvalidParity(v) => {
                    assert_eq!(
                        v as u8, data[64],
                        "alloy InvalidParity must surface the same trailing byte",
                    );
                }
                other => panic!("alloy rejected with non-parity error: {other:?}"),
            }
        }
        (Ok(_), Err(alloy_err)) => {
            panic!(
                "cow accepted but alloy rejected (cow must be a strict subset of alloy): {alloy_err:?}",
            );
        }
        (Err(other), _) => {
            panic!("cow rejected through unexpected variant on 65-byte fixed input: {other:?}",)
        }
    }
});
