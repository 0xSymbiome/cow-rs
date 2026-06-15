//! Pins the [`cow_sdk_core::SignerError`] classification surface that
//! `AlloyClientError` exposes to the signing crate. The umbrella
//! alloy adapter composes a local-key signer with an HTTP provider
//! and never routes wallet prompts, so no variant can represent an
//! EIP-1193 4xxx user rejection: every arm must return `None`. New
//! rejection-class variants must extend the trait impl alongside the
//! new variant so this contract starts pinning the new code.

use cow_sdk_alloy::AlloyClientError;
use cow_sdk_core::{Redacted, SignerError, TransportErrorClass};

#[test]
fn every_variant_returns_none_so_the_signer_helper_keeps_redaction() {
    let cases: [AlloyClientError; 7] = [
        AlloyClientError::Validation("validation-detail".to_owned()),
        AlloyClientError::Transport {
            class: TransportErrorClass::Timeout,
            detail: Redacted::new("transport-detail".to_owned()),
        },
        AlloyClientError::Remote {
            code: -32_000,
            message: "remote".to_owned(),
        },
        AlloyClientError::Signing {
            detail: Redacted::new("signing-detail".to_owned()),
        },
        AlloyClientError::PendingTransaction {
            detail: Redacted::new("pending-transaction-detail".to_owned()),
        },
        AlloyClientError::Cancelled,
        AlloyClientError::Internal("internal-detail".to_owned()),
    ];
    for error in cases {
        assert!(
            error.user_rejection_code().is_none(),
            "AlloyClientError must not classify as a user rejection: {error:?}"
        );
    }
}
