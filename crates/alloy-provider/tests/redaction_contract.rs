use cow_sdk_alloy_provider::{
    AsyncProviderError, AsyncProviderErrorClass, RpcAlloyProvider, RpcAlloyProviderBuilderError,
};
use cow_sdk_core::{Redacted, TransportErrorClass};

const SECRET_URL: &str = "https://user:secret@example.invalid/rpc?api_key=top-secret";

#[test]
fn validation_display_does_not_leak_url() {
    let error = AsyncProviderError::Validation(SECRET_URL.to_owned());

    assert_no_secret(&format!("{error}"));
    assert_no_secret(&format!("{error:?}"));
}

#[test]
fn transport_display_redacts_detail() {
    let error = AsyncProviderError::Transport {
        class: TransportErrorClass::Other,
        detail: Redacted::new(SECRET_URL.to_owned()),
    };

    assert!(format!("{error}").contains("[redacted]"));
    assert_no_secret(&format!("{error}"));
    assert_no_secret(&format!("{error:?}"));
}

#[test]
fn transport_source_chain_does_not_leak_url() {
    let error = AsyncProviderError::Transport {
        class: TransportErrorClass::Other,
        detail: Redacted::new(SECRET_URL.to_owned()),
    };

    let source = std::error::Error::source(&error);

    assert!(source.is_none());
}

#[test]
fn remote_display_emits_code_and_message() {
    let error = AsyncProviderError::Remote {
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
    let error = AsyncProviderError::Internal(SECRET_URL.to_owned());

    assert_no_secret(&format!("{error}"));
    assert_no_secret(&format!("{error:?}"));
}

#[test]
fn error_class_covers_every_variant() {
    assert_eq!(
        AsyncProviderError::Validation("bad input".to_owned()).class(),
        AsyncProviderErrorClass::Validation
    );
    assert_eq!(
        AsyncProviderError::Transport {
            class: TransportErrorClass::Other,
            detail: Redacted::new("transport".to_owned()),
        }
        .class(),
        AsyncProviderErrorClass::Transport
    );
    assert_eq!(
        AsyncProviderError::Remote {
            code: -32000,
            message: "execution reverted".to_owned(),
        }
        .class(),
        AsyncProviderErrorClass::Remote
    );
    assert_eq!(
        AsyncProviderError::Cancelled.class(),
        AsyncProviderErrorClass::Cancelled
    );
    assert_eq!(
        AsyncProviderError::Internal("internal".to_owned()).class(),
        AsyncProviderErrorClass::Internal
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
        .await
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
