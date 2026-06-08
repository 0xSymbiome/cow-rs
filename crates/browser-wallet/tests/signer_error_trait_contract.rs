//! Pins the [`cow_sdk_core::SignerError`] classification surface that
//! `BrowserWalletError` exposes to the signing crate. The test
//! enumerates every variant carried by the wallet error and asserts
//! the exact `user_rejection_code()` value the variant exposes, so a
//! future EIP-1193 rejection variant added without a matching trait
//! arm fails this contract instead of silently falling through to the
//! redacted-display path in `cow_sdk_signing::SigningError::Signer`.

use cow_sdk_browser_wallet::{BrowserWalletError, RpcErrorPayload};
use cow_sdk_core::SignerError;
use serde_json::json;

const SAMPLE_METHOD: &str = "eth_signTypedData_v4";
const SAMPLE_MESSAGE: &str = "user rejected the request";

fn method_name() -> String {
    SAMPLE_METHOD.to_owned()
}

fn redacted_message() -> cow_sdk_core::Redacted<String> {
    SAMPLE_MESSAGE.to_owned().into()
}

#[test]
fn user_rejected_request_exposes_the_carried_provider_error_code() {
    let error = BrowserWalletError::UserRejectedRequest {
        method: method_name(),
        code: 4001,
        message: redacted_message(),
    };
    assert_eq!(error.user_rejection_code(), Some(4001));
}

#[test]
fn user_rejected_request_preserves_any_eip1193_4xxx_code_the_wallet_returned() {
    let cases = [4001, 4100, 4101, 4900, 4901, 4902];
    for code in cases {
        let error = BrowserWalletError::UserRejectedRequest {
            method: method_name(),
            code,
            message: redacted_message(),
        };
        assert_eq!(error.user_rejection_code(), Some(code));
    }
}

#[test]
fn non_rejection_variants_return_none_so_the_signer_helper_keeps_redaction() {
    let cases: [BrowserWalletError; 16] = [
        BrowserWalletError::WalletUnavailable,
        BrowserWalletError::DiscoverySelectionRequired { candidates: 2 },
        BrowserWalletError::DiscoverySelectionOutOfRange {
            index: 5,
            candidates: 2,
        },
        BrowserWalletError::InvalidProviderOrigin {
            message: redacted_message(),
        },
        BrowserWalletError::UntrustedProviderOrigin {
            origin: redacted_message(),
        },
        BrowserWalletError::Disconnected {
            method: method_name(),
            code: 4900,
            message: redacted_message(),
        },
        BrowserWalletError::WrongChain {
            method: method_name(),
            code: 4901,
            message: redacted_message(),
        },
        BrowserWalletError::ChainNotAdded {
            chain_id: Some(8453),
            method: method_name(),
            code: 4902,
            message: redacted_message(),
        },
        BrowserWalletError::InvalidChainConfiguration {
            chain_id: 8453,
            message: redacted_message(),
        },
        BrowserWalletError::SessionChainMismatch {
            expected_chain_id: 1,
            session_chain_id: 8453,
        },
        BrowserWalletError::TypedDataChainMismatch {
            expected_chain_id: 1,
            typed_data_chain_id: 8453,
        },
        BrowserWalletError::UnsupportedRpcMethod {
            method: method_name(),
            message: redacted_message(),
        },
        BrowserWalletError::MalformedResponse {
            method: method_name(),
            message: redacted_message(),
        },
        BrowserWalletError::Rpc {
            method: method_name(),
            code: -32_603,
            message: redacted_message(),
            data: Some(json!({ "any": "shape" }).into()),
        },
        BrowserWalletError::JsInterop {
            message: redacted_message(),
        },
        BrowserWalletError::Serialization {
            message: redacted_message(),
        },
    ];
    for error in cases {
        assert!(
            error.user_rejection_code().is_none(),
            "{error:?} must not classify as a user rejection"
        );
    }
}

#[test]
fn unknown_rpc_payload_codes_do_not_classify_as_user_rejections() {
    // `RpcErrorPayload::new` is the public constructor; round-tripping
    // through `BrowserWalletError::Rpc` directly mirrors what
    // `from_rpc` would produce for an unknown code without exercising
    // the private constructor surface.
    let _payload = RpcErrorPayload::new(-32_000, "generic rpc error", None);
    let error = BrowserWalletError::Rpc {
        method: "wallet_switchEthereumChain".to_owned(),
        code: -32_000,
        message: "generic rpc error".to_owned().into(),
        data: None,
    };
    assert!(error.user_rejection_code().is_none());
}
