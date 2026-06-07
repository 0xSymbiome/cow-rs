mod common;

use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;
use cow_sdk_app_data::{
    AppDataError, DEFAULT_IPFS_READ_URI, IpfsConfig, IpfsFetchPolicy, IpfsFetchTransport,
    fetch_doc_from_app_data_hex, fetch_doc_from_cid, fetch_doc_from_cid_with_policy,
};
use serde_json::Value;

use crate::common::{APP_DATA_HEX, APP_DATA_HEX_2, APP_DATA_STRING, APP_DATA_STRING_2, CID, CID_2};

#[derive(Default)]
struct RecordingFetchTransport {
    responses: HashMap<String, String>,
    captured: Mutex<Vec<String>>,
}

impl RecordingFetchTransport {
    fn with_response(mut self, uri: &str, body: &str) -> Self {
        self.responses.insert(uri.to_string(), body.to_string());
        self
    }

    fn requests(&self) -> Vec<String> {
        self.captured.lock().unwrap().clone()
    }
}

#[async_trait]
impl IpfsFetchTransport for RecordingFetchTransport {
    async fn get(&self, uri: &str) -> Result<String, AppDataError> {
        self.captured.lock().unwrap().push(uri.to_string());
        self.responses
            .get(uri)
            .cloned()
            .ok_or_else(|| AppDataError::Transport {
                class: cow_sdk_core::TransportErrorClass::Other,
                detail: format!("missing fixture for {uri}").into(),
            })
    }
}

#[tokio::test]
async fn async_trait_fetches_cid_with_default_ipfs_uri() {
    let transport = RecordingFetchTransport::default().with_response(
        &format!("https://gnosis.mypinata.cloud/ipfs/{CID}"),
        APP_DATA_STRING,
    );

    let from_cid = fetch_doc_from_cid(CID, &transport, None).await.unwrap();
    assert_eq!(
        from_cid,
        serde_json::from_str::<Value>(APP_DATA_STRING).unwrap()
    );
    assert_eq!(
        transport.requests(),
        vec![format!("https://gnosis.mypinata.cloud/ipfs/{CID}")]
    );
}

#[tokio::test]
async fn async_trait_fetches_doc_from_app_data_hex() {
    let transport = RecordingFetchTransport::default().with_response(
        &format!("https://gnosis.mypinata.cloud/ipfs/{CID}"),
        APP_DATA_STRING,
    );

    let from_hex = fetch_doc_from_app_data_hex(APP_DATA_HEX, &transport, None)
        .await
        .unwrap();

    assert_eq!(
        from_hex,
        serde_json::from_str::<Value>(APP_DATA_STRING).unwrap()
    );
    assert_eq!(
        transport.requests(),
        vec![format!("https://gnosis.mypinata.cloud/ipfs/{CID}")]
    );
}

#[tokio::test]
async fn fetch_by_app_data_hex_rejects_invalid_hex() {
    let transport = RecordingFetchTransport::default();
    let error = fetch_doc_from_app_data_hex("invalidHash", &transport, None)
        .await
        .unwrap_err();
    match &error {
        AppDataError::Transport { detail, .. } => {
            assert!(detail.as_inner().contains("error decoding appDataHex"));
        }
        other => panic!("expected Transport error, got {other:?}"),
    }
    assert_eq!(error.to_string(), "transport error (decode): [redacted]");
}

#[test]
fn fetch_policy_defaults_and_trims_explicit_read_base_urls() {
    let default_policy = IpfsFetchPolicy::default();
    let explicit_policy =
        IpfsFetchPolicy::new("https://ipfs.example.test/ipfs/").expect("policy should normalize");

    assert_eq!(
        default_policy.read_base_uri(),
        "https://gnosis.mypinata.cloud/ipfs"
    );
    assert_eq!(
        explicit_policy.read_base_uri(),
        "https://ipfs.example.test/ipfs"
    );
}

/// Pins the default IPFS read gateway to the upstream `@cowprotocol/config`
/// value. App-data reads resolve keccak-256 `CIDv1` documents, which a generic
/// public gateway cannot serve, so an upstream gateway change must land here in
/// lockstep rather than surface as a silent read failure for consumers.
#[test]
fn default_ipfs_read_gateway_tracks_upstream_config() {
    assert_eq!(
        DEFAULT_IPFS_READ_URI, "https://gnosis.mypinata.cloud/ipfs",
        "DEFAULT_IPFS_READ_URI must mirror @cowprotocol/config; update both together",
    );
}

#[test]
fn fetch_policy_rejects_empty_base_uri() {
    let error = IpfsFetchPolicy::new("   ").expect_err("empty base URI must fail");
    assert!(matches!(error, AppDataError::Transport { class, .. }
        if class == cow_sdk_core::TransportErrorClass::Builder));
}

#[test]
fn fetch_policy_can_be_derived_from_read_config() {
    let config = IpfsConfig {
        uri: Some("https://fallback.example.test/ipfs".to_owned().into()),
        read_uri: Some("https://read.example.test/ipfs/".to_owned().into()),
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

#[tokio::test]
async fn fetch_helpers_accept_typed_policy_and_custom_read_base_uri() {
    let policy =
        IpfsFetchPolicy::new("https://ipfs.example.test/ipfs").expect("policy should be valid");
    let transport = RecordingFetchTransport::default().with_response(
        &format!("https://ipfs.example.test/ipfs/{CID}"),
        APP_DATA_STRING,
    );

    let from_cid = fetch_doc_from_cid_with_policy(CID, &transport, &policy)
        .await
        .unwrap();

    assert_eq!(
        from_cid,
        serde_json::from_str::<Value>(APP_DATA_STRING).unwrap()
    );
    assert_eq!(
        transport.requests(),
        vec![format!("https://ipfs.example.test/ipfs/{CID}")]
    );
}

#[tokio::test]
async fn fetch_helpers_keep_distinct_cid_requests() {
    let transport = RecordingFetchTransport::default()
        .with_response(
            &format!("https://gnosis.mypinata.cloud/ipfs/{CID}"),
            APP_DATA_STRING,
        )
        .with_response(
            &format!("https://gnosis.mypinata.cloud/ipfs/{CID_2}"),
            APP_DATA_STRING_2,
        );

    let first = fetch_doc_from_app_data_hex(APP_DATA_HEX, &transport, None)
        .await
        .unwrap();
    let second = fetch_doc_from_app_data_hex(APP_DATA_HEX_2, &transport, None)
        .await
        .unwrap();

    assert_ne!(first, second);
    assert_eq!(
        transport.requests(),
        vec![
            format!("https://gnosis.mypinata.cloud/ipfs/{CID}"),
            format!("https://gnosis.mypinata.cloud/ipfs/{CID_2}")
        ]
    );
}

#[tokio::test]
async fn fetch_doc_from_cid_with_policy_rejects_malformed_json() {
    let policy = IpfsFetchPolicy::default();
    let transport = RecordingFetchTransport::default().with_response(
        &format!("https://gnosis.mypinata.cloud/ipfs/{CID}"),
        "not-json",
    );

    let error = fetch_doc_from_cid_with_policy(CID, &transport, &policy)
        .await
        .expect_err("malformed json must fail");

    assert!(matches!(error, AppDataError::Json { .. }));
}

#[tokio::test]
async fn fetch_doc_from_cid_rejects_empty_explicit_read_base_uri() {
    let transport = RecordingFetchTransport::default();
    let error = fetch_doc_from_cid(CID, &transport, Some("   "))
        .await
        .expect_err("blank policy override must fail");

    match &error {
        AppDataError::Transport { detail, .. } => {
            assert_eq!(detail.as_inner(), "ipfs read base uri must not be empty");
        }
        other => panic!("expected Transport error, got {other:?}"),
    }
    assert_eq!(error.to_string(), "transport error (builder): [redacted]");
}

#[tokio::test]
async fn missing_fixture_maps_to_transport_error() {
    let error = fetch_doc_from_cid(CID, &RecordingFetchTransport::default(), None)
        .await
        .expect_err("missing fixture should fail through transport");

    assert_eq!(error.to_string(), "transport error (other): [redacted]");
}
