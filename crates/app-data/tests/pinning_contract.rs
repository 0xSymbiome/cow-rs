mod common;

use std::cell::RefCell;

use cow_sdk_app_data::{
    AppDataError, AppDataParams, IpfsConfig, IpfsUploadTransport, TransportResponse,
    generate_app_data_doc, pin_json_in_pinata_ipfs,
};
use serde_json::json;

use crate::common::PINATA_IPFS_HASH;

type UploadHeaders = Vec<(String, String)>;
type UploadRequest = (String, String, UploadHeaders);

#[derive(Default)]
struct RecordingUploadTransport {
    response: RefCell<Option<TransportResponse>>,
    requests: RefCell<Vec<UploadRequest>>,
}

impl RecordingUploadTransport {
    fn with_response(self, response: TransportResponse) -> Self {
        self.response.replace(Some(response));
        self
    }

    fn request(&self) -> Option<UploadRequest> {
        self.requests.borrow().last().cloned()
    }

    fn requests(&self) -> Vec<UploadRequest> {
        self.requests.borrow().clone()
    }
}

impl IpfsUploadTransport for RecordingUploadTransport {
    fn post_json(
        &self,
        uri: &str,
        body: &str,
        headers: &[(String, String)],
    ) -> Result<TransportResponse, AppDataError> {
        self.requests
            .borrow_mut()
            .push((uri.to_string(), body.to_string(), headers.to_vec()));
        self.response
            .borrow()
            .clone()
            .ok_or_else(|| AppDataError::Transport {
                class: cow_sdk_core::TransportErrorClass::Other,
                detail: "missing upload response".to_string(),
            })
    }
}

fn credentialed_config_with_write_uri(write_uri: Option<&str>) -> IpfsConfig {
    IpfsConfig {
        write_uri: write_uri.map(str::to_owned),
        pinata_api_key: Some("apikey".to_string().into()),
        pinata_api_secret: Some("apiSecret".to_string().into()),
        ..IpfsConfig::default()
    }
}

fn pinata_success_transport() -> RecordingUploadTransport {
    RecordingUploadTransport::default().with_response(TransportResponse {
        status: 200,
        body: format!("{{\"IpfsHash\":\"{PINATA_IPFS_HASH}\"}}"),
    })
}

fn assert_write_uri_rejected_with_builder_error(write_uri: &str) {
    let transport = RecordingUploadTransport::default();
    let document = generate_app_data_doc(AppDataParams::default());
    let config = credentialed_config_with_write_uri(Some(write_uri));

    let error = pin_json_in_pinata_ipfs(&document, &transport, &config).unwrap_err();

    match error {
        AppDataError::Transport {
            class: cow_sdk_core::TransportErrorClass::Builder,
            ref detail,
        } => {
            assert_eq!(detail, "ipfs write base uri must not be empty");
        }
        other => panic!("expected Transport(Builder), got {other:?}"),
    }

    assert!(
        transport.requests().is_empty(),
        "transport must not be called when write_uri preflight fails",
    );
}

#[test]
fn pinning_requires_explicit_credentials() {
    let transport = RecordingUploadTransport::default();
    let document = generate_app_data_doc(AppDataParams::default());
    let error = pin_json_in_pinata_ipfs(&document, &transport, &IpfsConfig::default()).unwrap_err();
    assert_eq!(error.to_string(), "You need to pass IPFS api credentials.");
}

#[test]
fn pinning_uses_deterministic_body_and_surfaces_the_returned_cid() {
    let transport = RecordingUploadTransport::default().with_response(TransportResponse {
        status: 200,
        body: format!("{{\"IpfsHash\":\"{PINATA_IPFS_HASH}\"}}"),
    });
    let document = generate_app_data_doc(AppDataParams {
        metadata: serde_json::from_value(json!({
            "referrer": { "code": "COWREF1" }
        }))
        .unwrap(),
        ..AppDataParams::default()
    });
    let config = IpfsConfig {
        pinata_api_key: Some("apikey".to_string().into()),
        pinata_api_secret: Some("apiSecret".to_string().into()),
        ..IpfsConfig::default()
    };

    let response = pin_json_in_pinata_ipfs(&document, &transport, &config).unwrap();

    assert_eq!(response["IpfsHash"].as_str(), Some(PINATA_IPFS_HASH));

    let (uri, body, headers) = transport.request().unwrap();
    assert_eq!(uri, "https://api.pinata.cloud/pinning/pinJSONToIPFS");
    assert_eq!(
        body,
        "{\"pinataContent\":{\"appCode\":\"CoW Swap\",\"metadata\":{\"referrer\":{\"code\":\"COWREF1\"}},\"version\":\"1.14.0\"},\"pinataMetadata\":{\"name\":\"appData\"}}"
    );
    assert!(headers.contains(&("Content-Type".to_string(), "application/json".to_string())));
    assert!(headers.contains(&("pinata_api_key".to_string(), "apikey".to_string())));
    assert!(headers.contains(&("pinata_secret_api_key".to_string(), "apiSecret".to_string())));
}

#[test]
fn write_uri_empty_returns_typed_builder_error() {
    assert_write_uri_rejected_with_builder_error("");
}

#[test]
fn write_uri_whitespace_returns_typed_builder_error() {
    assert_write_uri_rejected_with_builder_error("   ");
}

#[test]
fn write_uri_slash_only_returns_typed_builder_error() {
    assert_write_uri_rejected_with_builder_error("/");
}

#[test]
fn write_uri_trims_trailing_slash_and_succeeds() {
    let transport = pinata_success_transport();
    let document = generate_app_data_doc(AppDataParams::default());
    let config = credentialed_config_with_write_uri(Some("https://api.pinata.cloud/"));

    pin_json_in_pinata_ipfs(&document, &transport, &config).expect("upload succeeds");

    let (uri, _, _) = transport
        .request()
        .expect("transport is called exactly once");
    assert_eq!(uri, "https://api.pinata.cloud/pinning/pinJSONToIPFS");
    assert_eq!(transport.requests().len(), 1);
}

#[test]
fn write_uri_preserves_valid_input_unchanged() {
    let transport = pinata_success_transport();
    let document = generate_app_data_doc(AppDataParams::default());
    let config = credentialed_config_with_write_uri(Some("https://api.pinata.cloud"));

    pin_json_in_pinata_ipfs(&document, &transport, &config).expect("upload succeeds");

    let (uri, _, _) = transport
        .request()
        .expect("transport is called exactly once");
    assert_eq!(uri, "https://api.pinata.cloud/pinning/pinJSONToIPFS");
    assert_eq!(transport.requests().len(), 1);
}
