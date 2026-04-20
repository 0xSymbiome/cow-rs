mod common;

use std::{cell::RefCell, collections::HashMap};

use cow_sdk_app_data::{
    AppDataError, IpfsConfig, IpfsFetchPolicy, IpfsFetchTransport, fetch_doc_from_app_data_hex,
    fetch_doc_from_cid, fetch_doc_from_cid_with_policy,
};
use serde_json::Value;

use crate::common::{APP_DATA_HEX, APP_DATA_STRING, CID};

#[derive(Default)]
struct RecordingFetchTransport {
    responses: HashMap<String, String>,
    requests: RefCell<Vec<String>>,
}

impl RecordingFetchTransport {
    fn with_response(mut self, uri: &str, body: &str) -> Self {
        self.responses.insert(uri.to_string(), body.to_string());
        self
    }

    fn requests(&self) -> Vec<String> {
        self.requests.borrow().clone()
    }
}

impl IpfsFetchTransport for RecordingFetchTransport {
    fn get(&self, uri: &str) -> Result<String, AppDataError> {
        self.requests.borrow_mut().push(uri.to_string());
        self.responses
            .get(uri)
            .cloned()
            .ok_or_else(|| AppDataError::Transport(format!("missing fixture for {uri}")))
    }
}

#[test]
fn fetch_helpers_use_explicit_transport_and_default_ipfs_uri() {
    let transport = RecordingFetchTransport::default().with_response(
        &format!("https://cloudflare-ipfs.com/ipfs/{CID}"),
        APP_DATA_STRING,
    );

    let from_cid = fetch_doc_from_cid(CID, &transport, None).unwrap();
    let from_hex = fetch_doc_from_app_data_hex(APP_DATA_HEX, &transport, None).unwrap();

    assert_eq!(
        from_cid,
        serde_json::from_str::<Value>(APP_DATA_STRING).unwrap()
    );
    assert_eq!(
        from_hex,
        serde_json::from_str::<Value>(APP_DATA_STRING).unwrap()
    );
    assert_eq!(
        transport.requests(),
        vec![
            format!("https://cloudflare-ipfs.com/ipfs/{CID}"),
            format!("https://cloudflare-ipfs.com/ipfs/{CID}")
        ]
    );
}

#[test]
fn fetch_by_app_data_hex_rejects_invalid_hex() {
    let transport = RecordingFetchTransport::default();
    let error = fetch_doc_from_app_data_hex("invalidHash", &transport, None).unwrap_err();
    assert!(matches!(error, AppDataError::Transport(_)));
    assert!(error.to_string().contains("Error decoding AppData"));
}

#[test]
fn fetch_policy_defaults_and_trims_explicit_read_base_urls() {
    let default_policy = IpfsFetchPolicy::default();
    let explicit_policy =
        IpfsFetchPolicy::new("https://ipfs.example.test/ipfs/").expect("policy should normalize");

    assert_eq!(
        default_policy.read_base_uri(),
        "https://cloudflare-ipfs.com/ipfs"
    );
    assert_eq!(
        explicit_policy.read_base_uri(),
        "https://ipfs.example.test/ipfs"
    );
}

#[test]
fn fetch_policy_can_be_derived_without_leaking_pinning_credentials() {
    let config = IpfsConfig {
        uri: Some("https://fallback.example.test/ipfs".to_owned()),
        read_uri: Some("https://read.example.test/ipfs/".to_owned()),
        write_uri: Some("https://write.example.test".to_owned()),
        pinata_api_key: Some("pinata-key".to_owned().into()),
        pinata_api_secret: Some("pinata-secret".to_owned().into()),
    };
    let policy = IpfsFetchPolicy::from_config(&config).expect("read config should be valid");

    assert_eq!(policy.read_base_uri(), "https://read.example.test/ipfs");
}

#[test]
fn fetch_policy_with_read_base_uri_replaces_the_existing_policy_value() {
    let policy = IpfsFetchPolicy::new("https://first.example.test/ipfs")
        .unwrap()
        .with_read_base_uri("https://second.example.test/ipfs/")
        .unwrap();

    assert_eq!(policy.read_base_uri(), "https://second.example.test/ipfs");
}

#[test]
fn fetch_helpers_accept_typed_policy_and_custom_read_base_uri() {
    let policy =
        IpfsFetchPolicy::new("https://ipfs.example.test/ipfs").expect("policy should be valid");
    let transport = RecordingFetchTransport::default().with_response(
        &format!("https://ipfs.example.test/ipfs/{CID}"),
        APP_DATA_STRING,
    );

    let from_cid = fetch_doc_from_cid_with_policy(CID, &transport, &policy).unwrap();

    assert_eq!(
        from_cid,
        serde_json::from_str::<Value>(APP_DATA_STRING).unwrap()
    );
    assert_eq!(
        transport.requests(),
        vec![format!("https://ipfs.example.test/ipfs/{CID}")]
    );
}

#[test]
fn fetch_doc_from_cid_with_policy_rejects_malformed_json() {
    let policy = IpfsFetchPolicy::default();
    let transport = RecordingFetchTransport::default().with_response(
        &format!("https://cloudflare-ipfs.com/ipfs/{CID}"),
        "not-json",
    );

    let error = fetch_doc_from_cid_with_policy(CID, &transport, &policy)
        .expect_err("malformed json must fail");

    assert!(matches!(error, AppDataError::Json(_)));
}

#[test]
fn fetch_doc_from_cid_rejects_empty_explicit_read_base_uri() {
    let transport = RecordingFetchTransport::default();
    let error = fetch_doc_from_cid(CID, &transport, Some("   "))
        .expect_err("blank policy override must fail");

    assert_eq!(
        error.to_string(),
        "transport error: ipfs read base uri must not be empty"
    );
}
