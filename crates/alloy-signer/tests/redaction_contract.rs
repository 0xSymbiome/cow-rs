#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy_signer::{SignerError, SignerErrorClass, LocalAlloyKeystoreSigner};
use cow_sdk_core::{Redacted, SupportedChainId};

#[test]
fn validation_display_and_debug_do_not_leak_input() {
    let secret = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
    let error = SignerError::Validation(secret.to_owned());

    assert_redacted(&error, secret);
    assert_eq!(error.class(), SignerErrorClass::Validation);
}

#[test]
fn signing_display_and_debug_redact_detail() {
    let secret = "raw signer backend included private material";
    let error = SignerError::Signing {
        detail: Redacted::new(secret.to_owned()),
    };

    assert_redacted(&error, secret);
    assert_eq!(error.class(), SignerErrorClass::Signing);
}

#[test]
fn provider_required_includes_method_only() {
    let error = SignerError::ProviderRequired {
        method: "send_transaction",
    };

    let display = error.to_string();
    let debug = format!("{error:?}");
    assert!(display.contains("send_transaction"));
    assert!(debug.contains("send_transaction"));
    assert!(!display.contains("private"));
    assert_eq!(error.class(), SignerErrorClass::ProviderRequired);
}

#[test]
fn unsupported_message_is_static() {
    let error = SignerError::Unsupported("typed data disabled");

    assert!(error.to_string().contains("typed data disabled"));
    assert_eq!(error.class(), SignerErrorClass::Unsupported);
}

#[test]
fn internal_display_and_debug_do_not_leak_input() {
    let secret = "internal private key fragment";
    let error = SignerError::Internal(secret.to_owned());

    assert_redacted(&error, secret);
    assert_eq!(error.class(), SignerErrorClass::Internal);
}

#[test]
fn builder_invalid_private_key_does_not_echo_bytes() {
    let secret = "not-a-real-private-key";
    let Err(error) = LocalAlloyKeystoreSigner::builder().private_key(secret) else {
        panic!("invalid key must fail");
    };

    let display = error.to_string();
    let debug = format!("{error:?}");
    assert!(!display.contains(secret));
    assert!(!debug.contains(secret));
    assert_eq!(display, "invalid private key");
}

#[test]
fn signer_debug_redacts_private_key_material() {
    let signer = LocalAlloyKeystoreSigner::builder()
        .private_key("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d")
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .unwrap();

    let debug = format!("{signer:?}");
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("59c6995e"));
}

#[test]
fn error_class_covers_every_variant() {
    let cases = [
        (
            SignerError::Validation("invalid".to_owned()),
            SignerErrorClass::Validation,
        ),
        (
            SignerError::Signing {
                detail: Redacted::new("secret".to_owned()),
            },
            SignerErrorClass::Signing,
        ),
        (
            SignerError::ProviderRequired { method: "send" },
            SignerErrorClass::ProviderRequired,
        ),
        (
            SignerError::Unsupported("unsupported"),
            SignerErrorClass::Unsupported,
        ),
        (
            SignerError::Cancelled,
            SignerErrorClass::Cancelled,
        ),
        (
            SignerError::Internal("internal".to_owned()),
            SignerErrorClass::Internal,
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.class(), expected);
    }
}

fn assert_redacted(error: &SignerError, secret: &str) {
    let display = error.to_string();
    let debug = format!("{error:?}");
    assert!(!display.contains(secret), "{display}");
    assert!(!debug.contains(secret), "{debug}");
    assert!(display.contains("[redacted]") || debug.contains("[redacted]"));

    let mut current = std::error::Error::source(error);
    while let Some(source) = current {
        let source_display = source.to_string();
        let source_debug = format!("{source:?}");
        assert!(!source_display.contains(secret), "{source_display}");
        assert!(!source_debug.contains(secret), "{source_debug}");
        current = source.source();
    }
}
