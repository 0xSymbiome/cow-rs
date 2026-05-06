#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::{AlloyClient, AlloyClientError, AlloyClientErrorClass};
use cow_sdk_core::{AsyncSigningProvider, Redacted, SupportedChainId, TransportErrorClass};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const SECRET_URL: &str = "https://user:secret@example.invalid/rpc?api_key=top-secret";
const SECRET_DETAIL: &str =
    "https://user:secret@example.invalid Authorization: Bearer token private-key-fragment";

#[test]
fn validation_display_and_debug_do_not_leak_input() {
    let error = AlloyClientError::Validation(SECRET_DETAIL.to_owned());

    assert_redacted(&error, SECRET_DETAIL);
    assert_eq!(error.class(), AlloyClientErrorClass::Validation);
}

#[test]
fn transport_display_and_debug_redact_detail() {
    let error = AlloyClientError::Transport {
        class: TransportErrorClass::Other,
        detail: Redacted::new(SECRET_DETAIL.to_owned()),
    };

    assert_redacted(&error, SECRET_DETAIL);
    assert_eq!(error.class(), AlloyClientErrorClass::Transport);
}

#[test]
fn remote_display_emits_code_and_safe_message() {
    let error = AlloyClientError::Remote {
        code: -32_000,
        message: "execution reverted".to_owned(),
    };

    assert_eq!(
        error.to_string(),
        "remote error (code -32000): execution reverted"
    );
    assert_eq!(error.class(), AlloyClientErrorClass::Remote);
}

#[test]
fn signing_display_and_debug_redact_detail() {
    let error = AlloyClientError::Signing {
        detail: Redacted::new(SECRET_DETAIL.to_owned()),
    };

    assert_redacted(&error, SECRET_DETAIL);
    assert_eq!(error.class(), AlloyClientErrorClass::Signing);
}

#[test]
fn pending_transaction_display_and_debug_redact_detail() {
    let error = AlloyClientError::PendingTransaction {
        detail: Redacted::new(SECRET_DETAIL.to_owned()),
    };

    assert_redacted(&error, SECRET_DETAIL);
    assert_eq!(error.class(), AlloyClientErrorClass::PendingTransaction);
}

#[test]
fn unsupported_transaction_request_uses_static_diagnostic() {
    let error = AlloyClientError::UnsupportedTransactionRequest {
        method: "sign_transaction",
        reason: "raw transaction signing is deferred; use send_transaction for on-chain operations",
    };
    let display = error.to_string();
    let debug = format!("{error:?}");

    assert!(display.contains("sign_transaction"));
    assert!(debug.contains("sign_transaction"));
    assert_no_secret(&display, SECRET_DETAIL);
    assert_no_secret(&debug, SECRET_DETAIL);
    assert_eq!(
        error.class(),
        AlloyClientErrorClass::UnsupportedTransactionRequest
    );
}

#[test]
fn internal_display_and_debug_do_not_leak_input() {
    let error = AlloyClientError::Internal(SECRET_DETAIL.to_owned());

    assert_redacted(&error, SECRET_DETAIL);
    assert_eq!(error.class(), AlloyClientErrorClass::Internal);
}

#[test]
fn builder_invalid_inputs_do_not_echo_values() {
    let Err(url_error) = AlloyClient::builder().http("not a url with secret=top-secret") else {
        panic!("invalid URL must fail");
    };
    let Err(key_error) = AlloyClient::builder().private_key("not-a-private-key-fragment") else {
        panic!("invalid key must fail");
    };

    assert_no_secret(&url_error.to_string(), "top-secret");
    assert_no_secret(&format!("{url_error:?}"), "top-secret");
    assert_no_secret(&key_error.to_string(), "private-key-fragment");
    assert_no_secret(&format!("{key_error:?}"), "private-key-fragment");
}

#[tokio::test]
async fn client_and_handle_debug_redact_transport_and_key_material() {
    let client = AlloyClient::builder()
        .http(SECRET_URL)
        .unwrap()
        .private_key(TEST_KEY)
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .await
        .unwrap();
    let handle = client.create_signer("local-key").await.unwrap();

    let client_debug = format!("{client:?}");
    let handle_debug = format!("{handle:?}");

    assert!(client_debug.contains("<redacted>"));
    assert!(handle_debug.contains("[redacted]"));
    assert_no_secret(&client_debug, "top-secret");
    assert_no_secret(&client_debug, "user:secret");
    assert_no_secret(&handle_debug, "59c6995e");
}

#[test]
fn error_class_covers_every_variant() {
    let cases = [
        (
            AlloyClientError::Validation("invalid".to_owned()),
            AlloyClientErrorClass::Validation,
        ),
        (
            AlloyClientError::Transport {
                class: TransportErrorClass::Other,
                detail: Redacted::new("transport".to_owned()),
            },
            AlloyClientErrorClass::Transport,
        ),
        (
            AlloyClientError::Remote {
                code: -32_000,
                message: "execution reverted".to_owned(),
            },
            AlloyClientErrorClass::Remote,
        ),
        (
            AlloyClientError::Signing {
                detail: Redacted::new("signing".to_owned()),
            },
            AlloyClientErrorClass::Signing,
        ),
        (
            AlloyClientError::PendingTransaction {
                detail: Redacted::new("pending".to_owned()),
            },
            AlloyClientErrorClass::PendingTransaction,
        ),
        (
            AlloyClientError::UnsupportedTransactionRequest {
                method: "sign_transaction",
                reason: "unsupported",
            },
            AlloyClientErrorClass::UnsupportedTransactionRequest,
        ),
        (
            AlloyClientError::Cancelled,
            AlloyClientErrorClass::Cancelled,
        ),
        (
            AlloyClientError::Internal("internal".to_owned()),
            AlloyClientErrorClass::Internal,
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.class(), expected);
        assert_eq!(error.class().to_string(), expected.as_str());
    }
}

fn assert_redacted(error: &AlloyClientError, secret: &str) {
    let display = error.to_string();
    let debug = format!("{error:?}");

    assert_no_secret(&display, secret);
    assert_no_secret(&debug, secret);
    assert!(display.contains("[redacted]") || debug.contains("[redacted]"));
}

fn assert_no_secret(rendered: &str, secret: &str) {
    assert!(
        !rendered.contains(secret),
        "secret substring leaked in {rendered:?}"
    );
}
