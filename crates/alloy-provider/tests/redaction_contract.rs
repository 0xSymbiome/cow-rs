use cow_sdk_alloy_provider::{
    ProviderError, ProviderErrorClass, RpcAlloyProvider, RpcAlloyProviderBuilderError,
};
use cow_sdk_core::{Redacted, TransportErrorClass};

const SECRET_URL: &str = "https://user:secret@example.invalid/rpc?api_key=top-secret";

#[test]
fn validation_display_does_not_leak_url() {
    let error = ProviderError::Validation(SECRET_URL.to_owned().into());

    assert_no_secret(&format!("{error}"));
    assert_no_secret(&format!("{error:?}"));
}

#[test]
fn transport_display_redacts_detail() {
    let error = ProviderError::Transport {
        class: TransportErrorClass::Other,
        detail: Redacted::new(SECRET_URL.to_owned()),
    };

    assert!(format!("{error}").contains("[redacted]"));
    assert_no_secret(&format!("{error}"));
    assert_no_secret(&format!("{error:?}"));
}

#[test]
fn transport_source_chain_does_not_leak_url() {
    let error = ProviderError::Transport {
        class: TransportErrorClass::Other,
        detail: Redacted::new(SECRET_URL.to_owned()),
    };

    let source = std::error::Error::source(&error);

    assert!(source.is_none());
}

#[test]
fn remote_display_emits_code_and_message() {
    let error = ProviderError::Remote {
        code: -32000,
        message: "execution reverted".to_owned(),
    };

    assert_eq!(
        format!("{error}"),
        "remote error (code -32000): execution reverted"
    );
}

#[test]
fn internal_display_does_not_leak_input() {
    let error = ProviderError::Internal(SECRET_URL.to_owned().into());

    assert_no_secret(&format!("{error}"));
    assert_no_secret(&format!("{error:?}"));
}

#[test]
fn error_class_covers_every_variant() {
    assert_eq!(
        ProviderError::Validation("bad input".to_owned().into()).class(),
        ProviderErrorClass::Validation
    );
    assert_eq!(
        ProviderError::Transport {
            class: TransportErrorClass::Other,
            detail: Redacted::new("transport".to_owned()),
        }
        .class(),
        ProviderErrorClass::Transport
    );
    assert_eq!(
        ProviderError::Remote {
            code: -32000,
            message: "execution reverted".to_owned(),
        }
        .class(),
        ProviderErrorClass::Remote
    );
    assert_eq!(
        ProviderError::Cancelled.class(),
        ProviderErrorClass::Cancelled
    );
    assert_eq!(
        ProviderError::Internal("internal".to_owned().into()).class(),
        ProviderErrorClass::Internal
    );
}

#[test]
fn builder_invalid_url_does_not_echo_input() {
    let error = RpcAlloyProvider::builder()
        .http("not a url with secret=top-secret")
        .unwrap_err();

    assert!(matches!(error, RpcAlloyProviderBuilderError::InvalidUrl));
    assert_no_secret(&format!("{error}"));
    assert_no_secret(&format!("{error:?}"));
}

#[tokio::test]
async fn provider_debug_redacts_credential_bearing_url() {
    let provider = RpcAlloyProvider::builder()
        .http(SECRET_URL)
        .unwrap()
        .build()
        .unwrap();

    let debug = format!("{provider:?}");
    assert!(debug.contains("chain_id"));
    assert!(debug.contains("transport"));
    assert!(debug.contains("<redacted>"));
    assert_no_secret(&debug);
}

fn assert_no_secret(value: &str) {
    assert!(!value.contains("top-secret"), "{value}");
    assert!(!value.contains("api_key"), "{value}");
    assert!(!value.contains("user:secret"), "{value}");
}
