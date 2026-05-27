//! Pins the [`cow_sdk_core::SignerError`] classification surface that
//! `SignerError` exposes to the signing crate. The
//! local-key alloy signer holds the private key in-process and never
//! produces an EIP-1193 wallet rejection, so every variant must
//! return `None` from `user_rejection_code()`. A future variant that
//! does represent a rejection (for example, an external-prompt
//! integration) must update the trait impl alongside the new variant
//! so this contract starts pinning the new code.

use cow_sdk_alloy_signer::SignerError;
use cow_sdk_core::{Redacted, SignerError as SignerErrorTrait};

#[test]
fn every_variant_returns_none_so_the_signer_helper_keeps_redaction() {
    let cases: [SignerError; 6] = [
        SignerError::Validation("validation-detail".to_owned()),
        SignerError::Signing {
            detail: Redacted::new("signing-detail".to_owned()),
        },
        SignerError::ProviderRequired {
            method: "send_transaction",
        },
        SignerError::Unsupported("unsupported-method"),
        SignerError::Cancelled,
        SignerError::Internal("internal-detail".to_owned()),
    ];
    for error in cases {
        assert!(
            SignerErrorTrait::user_rejection_code(&error).is_none(),
            "SignerError must not classify as a user rejection: {error:?}"
        );
    }
}
