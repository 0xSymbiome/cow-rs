//! Pins the [`cow_sdk_core::SignerError`] classification surface that
//! `AsyncSignerError` exposes to the signing crate. The
//! local-key alloy signer holds the private key in-process and never
//! produces an EIP-1193 wallet rejection, so every variant must
//! return `None` from `user_rejection_code()`. A future variant that
//! does represent a rejection (for example, an external-prompt
//! integration) must update the trait impl alongside the new variant
//! so this contract starts pinning the new code.

use cow_sdk_alloy_signer::AsyncSignerError;
use cow_sdk_core::{Redacted, SignerError};

#[test]
fn every_variant_returns_none_so_the_signer_helper_keeps_redaction() {
    let cases: [AsyncSignerError; 6] = [
        AsyncSignerError::Validation("validation-detail".to_owned()),
        AsyncSignerError::Signing {
            detail: Redacted::new("signing-detail".to_owned()),
        },
        AsyncSignerError::ProviderRequired {
            method: "send_transaction",
        },
        AsyncSignerError::Unsupported("unsupported-method"),
        AsyncSignerError::Cancelled,
        AsyncSignerError::Internal("internal-detail".to_owned()),
    ];
    for error in cases {
        assert!(
            error.user_rejection_code().is_none(),
            "AsyncSignerError must not classify as a user rejection: {error:?}"
        );
    }
}
