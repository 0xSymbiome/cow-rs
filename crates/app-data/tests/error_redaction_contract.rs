use std::cell::RefCell;

use cow_sdk_app_data::{
    AppDataError, AppDataParams, IpfsConfig, IpfsUploadTransport, TransportResponse,
    generate_app_data_doc, pin_json_in_pinata_ipfs,
};
use cow_sdk_core::{
    REDACTED_PLACEHOLDER, REDACTED_RESPONSE_BODY_MAX_BYTES, RESPONSE_BODY_TRUNCATION_MARKER,
    Redacted,
};
use serde_json::json;

const RAW_API_KEY: &str = "pinata-api-key-secret";
const RAW_TOKEN: &str = "pinata-token-secret";
const RAW_JWT: &str = "eyJhbGciOiJIUzI1NiJ9.eyJwaW5hdGEiOiJjb3cifQ.signature";

#[derive(Default)]
struct StubUploadTransport {
    response: RefCell<Option<TransportResponse>>,
}

impl StubUploadTransport {
    fn with_response(self, response: TransportResponse) -> Self {
        self.response.replace(Some(response));
        self
    }
}

impl IpfsUploadTransport for StubUploadTransport {
    fn post_json(
        &self,
        _uri: &str,
        _body: &str,
        _headers: &[(String, Redacted<String>)],
    ) -> Result<TransportResponse, AppDataError> {
        self.response
            .borrow()
            .clone()
            .ok_or_else(|| AppDataError::Transport {
                class: cow_sdk_core::TransportErrorClass::Other,
                detail: "missing upload response".to_owned(),
            })
    }
}

#[test]
fn pinning_error_body_is_redacted_at_storage_and_public_representations() {
    let body = json!({
        "error": {
            "details": format!(
                "Pinata rejected api_key={RAW_API_KEY}; token={RAW_TOKEN}; jwt={RAW_JWT}"
            )
        }
    });
    let transport =
        StubUploadTransport::default().with_response(TransportResponse::new(401, body.to_string()));
    let document = generate_app_data_doc(AppDataParams::default());
    let config = IpfsConfig {
        pinata_api_key: Some("configured-key".to_owned().into()),
        pinata_api_secret: Some("configured-secret".to_owned().into()),
        ..IpfsConfig::default()
    };

    let error = pin_json_in_pinata_ipfs(&document, &transport, &config)
        .expect_err("pinning status failures must surface through AppDataError::Pinning");

    let AppDataError::Pinning { message, .. } = &error else {
        panic!("expected Pinning error, got {error:?}");
    };
    assert_sanitized_storage(message.as_inner());
    assert_public_representations_are_redacted(&error);
}

fn assert_sanitized_storage(stored: &str) {
    assert!(stored.contains(REDACTED_PLACEHOLDER));
    assert!(
        stored.len() <= REDACTED_RESPONSE_BODY_MAX_BYTES + RESPONSE_BODY_TRUNCATION_MARKER.len()
    );
    assert_no_raw_credentials(stored);
}

fn assert_public_representations_are_redacted(error: &AppDataError) {
    let display = error.to_string();
    let compact_debug = format!("{error:?}");
    let pretty_debug = format!("{error:#?}");
    let json = serde_json::to_string(error).expect("AppDataError must serialize diagnostically");

    for rendered in [display, compact_debug, pretty_debug, json] {
        assert!(rendered.contains(REDACTED_PLACEHOLDER));
        assert_no_raw_credentials(&rendered);
    }
}

fn assert_no_raw_credentials(rendered: &str) {
    for forbidden in [RAW_API_KEY, RAW_TOKEN, RAW_JWT] {
        assert!(
            !rendered.contains(forbidden),
            "rendered output leaked {forbidden}: {rendered}"
        );
    }
}
