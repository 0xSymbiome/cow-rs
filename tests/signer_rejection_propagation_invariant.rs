//! End-to-end propagation invariant for the EIP-1193 4001
//! user-rejection classification.
//!
//! Drives the real `cow_sdk_signing::sign_order_async` against an
//! `AsyncTypedDataSigner` mock whose error type is the real
//! [`cow_sdk_browser_wallet::BrowserWalletError`] and asserts the
//! resulting [`cow_sdk_signing::SigningError`] is the typed
//! `SignerRejection { label, code }` variant — verifying every layer
//! between the wallet error variant and the SDK error surface
//! preserves the classification.
//!
//! The mock implements the same `AsyncTypedDataSigner` contract as
//! the production browser-wallet adapter (returning the same typed
//! error class on rejection) but stays pure-Rust so the invariant
//! exercises the trait wiring without a running browser. The
//! sibling host-test `crates/browser-wallet/tests/signer_error_trait_contract.rs`
//! pins the per-variant classification independently, and
//! `crates/signing/src/order_signing.rs` carries unit coverage for
//! the helper that routes the trait result into the
//! [`cow_sdk_signing::SigningError::SignerRejection`] variant.

use cow_sdk_browser_wallet::BrowserWalletError;
use cow_sdk_core::{
    Address, Amount, AppDataHash, AsyncTypedDataSigner, BuyTokenDestination, OrderKind,
    SellTokenSource, SupportedChainId, TypedDataPayload, UnsignedOrder,
};
use cow_sdk_signing::{SigningError, sign_order_async};

struct RejectingSigner {
    error: BrowserWalletError,
}

impl AsyncTypedDataSigner for RejectingSigner {
    type Error = BrowserWalletError;

    async fn sign_typed_data_payload(
        &self,
        _payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        Err(self.error.clone())
    }

    async fn sign_typed_data(
        &self,
        _domain: &cow_sdk_core::TypedDataDomain,
        _fields: &[cow_sdk_core::TypedDataField],
        _value_json: &str,
    ) -> Result<String, Self::Error> {
        Err(self.error.clone())
    }
}

fn sample_order() -> UnsignedOrder {
    UnsignedOrder::new(
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        Address::new("0x4444444444444444444444444444444444444444").unwrap(),
        Amount::new("1000000000000000000").unwrap(),
        Amount::new("2000000000000000000").unwrap(),
        1_735_689_600,
        AppDataHash::new("0x337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df")
            .unwrap(),
        Amount::ZERO,
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    )
}

#[tokio::test]
async fn typed_data_signing_rejection_propagates_as_typed_signer_rejection() {
    let signer = RejectingSigner {
        error: BrowserWalletError::UserRejectedRequest {
            method: "eth_signTypedData_v4".to_owned().into(),
            code: 4001,
            message: "User rejected typed-data signature".to_owned().into(),
        },
    };
    let order = sample_order();
    let result = sign_order_async(&order, SupportedChainId::Mainnet, &signer, None).await;
    match result {
        Err(SigningError::SignerRejection { label, code }) => {
            assert_eq!(label, "typed-data signature");
            assert_eq!(code, 4001);
        }
        other => panic!(
            "expected SigningError::SignerRejection for a real BrowserWalletError::\
             UserRejectedRequest, got {other:?}"
        ),
    }
}

#[tokio::test]
async fn typed_data_signing_rejection_renders_user_facing_label_and_code() {
    let signer = RejectingSigner {
        error: BrowserWalletError::UserRejectedRequest {
            method: "eth_signTypedData_v4".to_owned().into(),
            code: 4001,
            message: "User rejected typed-data signature".to_owned().into(),
        },
    };
    let order = sample_order();
    let rendered = sign_order_async(&order, SupportedChainId::Mainnet, &signer, None)
        .await
        .expect_err("signing must fail when the signer rejects")
        .to_string();
    assert!(
        rendered.contains("User rejected typed-data signature"),
        "errorText contract requires the operation label substring; got `{rendered}`"
    );
    assert!(
        rendered.contains("(4001)"),
        "console JavaScript classifier requires the parenthesised EIP-1193 code; \
         got `{rendered}`"
    );
}

#[tokio::test]
async fn non_rejection_signer_errors_keep_the_redacted_display_path() {
    let signer = RejectingSigner {
        error: BrowserWalletError::Rpc {
            method: "eth_signTypedData_v4".to_owned().into(),
            code: -32_603,
            message: "internal error".to_owned().into(),
            data: None,
        },
    };
    let order = sample_order();
    let result = sign_order_async(&order, SupportedChainId::Mainnet, &signer, None).await;
    match result {
        Err(SigningError::Signer { operation, message }) => {
            assert_eq!(operation, "sign_typed_data_payload");
            assert!(
                message.as_inner().contains("[redacted]"),
                "non-rejection upstream errors stay redacted; got `{}`",
                message.as_inner()
            );
        }
        other => panic!("expected SigningError::Signer fall-back, got {other:?}"),
    }
}
