use serde_json::Value;

use crate::{
    AppDataDoc, AppDataError, DEFAULT_IPFS_READ_URI, app_data_hex_to_cid,
    app_data_hex_to_cid_legacy,
};

pub trait IpfsFetchTransport {
    fn get(&self, uri: &str) -> Result<String, AppDataError>;
}

pub fn fetch_doc_from_cid(
    cid: &str,
    transport: &impl IpfsFetchTransport,
    ipfs_uri: Option<&str>,
) -> Result<AppDataDoc, AppDataError> {
    let base = ipfs_uri
        .unwrap_or(DEFAULT_IPFS_READ_URI)
        .trim_end_matches('/');
    let raw = transport.get(&format!("{base}/{cid}"))?;
    serde_json::from_str::<Value>(&raw).map_err(|err| AppDataError::Json(err.to_string()))
}

pub fn fetch_doc_from_app_data_hex(
    app_data_hex: &str,
    transport: &impl IpfsFetchTransport,
    ipfs_uri: Option<&str>,
) -> Result<AppDataDoc, AppDataError> {
    fetch_doc_from_app_data_hex_inner(app_data_hex_to_cid, app_data_hex, transport, ipfs_uri)
}

pub fn fetch_doc_from_app_data_hex_legacy(
    app_data_hex: &str,
    transport: &impl IpfsFetchTransport,
    ipfs_uri: Option<&str>,
) -> Result<AppDataDoc, AppDataError> {
    fetch_doc_from_app_data_hex_inner(
        app_data_hex_to_cid_legacy,
        app_data_hex,
        transport,
        ipfs_uri,
    )
}

fn fetch_doc_from_app_data_hex_inner(
    hex_to_cid: fn(&str) -> Result<String, AppDataError>,
    app_data_hex: &str,
    transport: &impl IpfsFetchTransport,
    ipfs_uri: Option<&str>,
) -> Result<AppDataDoc, AppDataError> {
    let cid = hex_to_cid(app_data_hex).map_err(|err| {
        AppDataError::Transport(format!(
            "Error decoding AppData: appDataHex={app_data_hex}, message={err}"
        ))
    })?;
    fetch_doc_from_cid(&cid, transport, ipfs_uri)
}
