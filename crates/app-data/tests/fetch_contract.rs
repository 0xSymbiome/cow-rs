mod common;

use std::{cell::RefCell, collections::HashMap};

use cow_sdk_app_data::{
    AppDataError, IpfsFetchTransport, fetch_doc_from_app_data_hex,
    fetch_doc_from_app_data_hex_legacy, fetch_doc_from_cid,
};
use serde_json::Value;

use crate::common::{APP_DATA_HEX_LEGACY, APP_DATA_STRING, CID, CID_LEGACY};

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
    let transport = RecordingFetchTransport::default()
        .with_response("https://cloudflare-ipfs.com/ipfs/f01551b20337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df", APP_DATA_STRING)
        .with_response("https://cloudflare-ipfs.com/ipfs/QmSwrFbdFcryazEr361YmSwtGcN4uo4U5DKpzA4KbGxw4Q", APP_DATA_STRING);

    let from_cid = fetch_doc_from_cid(CID, &transport, None).unwrap();
    let from_hex_legacy =
        fetch_doc_from_app_data_hex_legacy(APP_DATA_HEX_LEGACY, &transport, None).unwrap();

    assert_eq!(
        from_cid,
        serde_json::from_str::<Value>(APP_DATA_STRING).unwrap()
    );
    assert_eq!(
        from_hex_legacy,
        serde_json::from_str::<Value>(APP_DATA_STRING).unwrap()
    );
    assert_eq!(
        transport.requests(),
        vec![
            format!("https://cloudflare-ipfs.com/ipfs/{CID}"),
            format!("https://cloudflare-ipfs.com/ipfs/{CID_LEGACY}")
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
