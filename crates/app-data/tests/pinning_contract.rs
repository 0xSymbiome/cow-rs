mod common;

use std::cell::RefCell;

use cow_sdk_app_data::{
    AppDataError, AppDataParams, IpfsConfig, IpfsUploadTransport, TransportResponse,
    generate_app_data_doc, upload_metadata_doc_to_ipfs_legacy,
};
use serde_json::json;

use crate::common::{PINATA_APP_DATA_HEX, PINATA_IPFS_HASH};

type UploadHeaders = Vec<(String, String)>;
type UploadRequest = (String, String, UploadHeaders);

#[derive(Default)]
struct RecordingUploadTransport {
    response: RefCell<Option<TransportResponse>>,
    request: RefCell<Option<UploadRequest>>,
}

impl RecordingUploadTransport {
    fn with_response(self, response: TransportResponse) -> Self {
        self.response.replace(Some(response));
        self
    }

    fn request(&self) -> Option<UploadRequest> {
        self.request.borrow().clone()
    }
}

impl IpfsUploadTransport for RecordingUploadTransport {
    fn post_json(
        &self,
        uri: &str,
        body: &str,
        headers: &[(String, String)],
    ) -> Result<TransportResponse, AppDataError> {
        self.request
            .borrow_mut()
            .replace((uri.to_string(), body.to_string(), headers.to_vec()));
        self.response
            .borrow()
            .clone()
            .ok_or_else(|| AppDataError::Transport("missing upload response".to_string()))
    }
}

#[test]
fn pinning_requires_explicit_credentials() {
    let transport = RecordingUploadTransport::default();
    let document = generate_app_data_doc(AppDataParams::default());
    let error = upload_metadata_doc_to_ipfs_legacy(&document, &transport, &IpfsConfig::default())
        .unwrap_err();
    assert_eq!(error.to_string(), "You need to pass IPFS api credentials.");
}

#[test]
fn legacy_pinning_uses_deterministic_body_and_returns_cid_and_digest() {
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
        pinata_api_key: Some("apikey".to_string()),
        pinata_api_secret: Some("apiSecret".to_string()),
        ..IpfsConfig::default()
    };

    let result = upload_metadata_doc_to_ipfs_legacy(&document, &transport, &config).unwrap();

    assert_eq!(result.cid, PINATA_IPFS_HASH);
    assert_eq!(result.app_data, PINATA_APP_DATA_HEX);

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
