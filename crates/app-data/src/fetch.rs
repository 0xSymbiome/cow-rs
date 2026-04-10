use serde_json::Value;

use crate::{
    AppDataDoc, AppDataError, DEFAULT_IPFS_READ_URI, IpfsConfig, app_data_hex_to_cid,
    app_data_hex_to_cid_legacy,
};

pub trait IpfsFetchTransport {
    fn get(&self, uri: &str) -> Result<String, AppDataError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpfsFetchPolicy {
    read_base_uri: String,
}

impl Default for IpfsFetchPolicy {
    fn default() -> Self {
        Self {
            read_base_uri: DEFAULT_IPFS_READ_URI.to_owned(),
        }
    }
}

impl IpfsFetchPolicy {
    pub fn new(read_base_uri: impl Into<String>) -> Result<Self, AppDataError> {
        Ok(Self {
            read_base_uri: normalize_read_base_uri(read_base_uri.into())?,
        })
    }

    pub fn from_config(config: &IpfsConfig) -> Result<Self, AppDataError> {
        let read_base_uri = config
            .read_uri
            .as_deref()
            .or(config.uri.as_deref())
            .unwrap_or(DEFAULT_IPFS_READ_URI);

        Self::new(read_base_uri)
    }

    pub fn read_base_uri(&self) -> &str {
        &self.read_base_uri
    }

    pub fn with_read_base_uri(
        mut self,
        read_base_uri: impl Into<String>,
    ) -> Result<Self, AppDataError> {
        self.read_base_uri = normalize_read_base_uri(read_base_uri.into())?;
        Ok(self)
    }
}

pub fn fetch_doc_from_cid(
    cid: &str,
    transport: &impl IpfsFetchTransport,
    ipfs_uri: Option<&str>,
) -> Result<AppDataDoc, AppDataError> {
    fetch_doc_from_cid_with_policy(cid, transport, &policy_from_optional_uri(ipfs_uri)?)
}

pub fn fetch_doc_from_cid_with_policy(
    cid: &str,
    transport: &impl IpfsFetchTransport,
    policy: &IpfsFetchPolicy,
) -> Result<AppDataDoc, AppDataError> {
    let raw = transport.get(&format!("{}/{}", policy.read_base_uri(), cid))?;
    serde_json::from_str::<Value>(&raw).map_err(|err| AppDataError::Json(err.to_string()))
}

pub fn fetch_doc_from_app_data_hex(
    app_data_hex: &str,
    transport: &impl IpfsFetchTransport,
    ipfs_uri: Option<&str>,
) -> Result<AppDataDoc, AppDataError> {
    fetch_doc_from_app_data_hex_with_policy(
        app_data_hex,
        transport,
        &policy_from_optional_uri(ipfs_uri)?,
    )
}

pub fn fetch_doc_from_app_data_hex_with_policy(
    app_data_hex: &str,
    transport: &impl IpfsFetchTransport,
    policy: &IpfsFetchPolicy,
) -> Result<AppDataDoc, AppDataError> {
    fetch_doc_from_app_data_hex_inner(app_data_hex_to_cid, app_data_hex, transport, policy)
}

pub fn fetch_doc_from_app_data_hex_legacy(
    app_data_hex: &str,
    transport: &impl IpfsFetchTransport,
    ipfs_uri: Option<&str>,
) -> Result<AppDataDoc, AppDataError> {
    fetch_doc_from_app_data_hex_legacy_with_policy(
        app_data_hex,
        transport,
        &policy_from_optional_uri(ipfs_uri)?,
    )
}

pub fn fetch_doc_from_app_data_hex_legacy_with_policy(
    app_data_hex: &str,
    transport: &impl IpfsFetchTransport,
    policy: &IpfsFetchPolicy,
) -> Result<AppDataDoc, AppDataError> {
    fetch_doc_from_app_data_hex_inner(app_data_hex_to_cid_legacy, app_data_hex, transport, policy)
}

fn fetch_doc_from_app_data_hex_inner(
    hex_to_cid: fn(&str) -> Result<String, AppDataError>,
    app_data_hex: &str,
    transport: &impl IpfsFetchTransport,
    policy: &IpfsFetchPolicy,
) -> Result<AppDataDoc, AppDataError> {
    let cid = hex_to_cid(app_data_hex).map_err(|err| {
        AppDataError::Transport(format!(
            "Error decoding AppData: appDataHex={app_data_hex}, message={err}"
        ))
    })?;
    fetch_doc_from_cid_with_policy(&cid, transport, policy)
}

fn policy_from_optional_uri(ipfs_uri: Option<&str>) -> Result<IpfsFetchPolicy, AppDataError> {
    match ipfs_uri {
        Some(read_base_uri) => IpfsFetchPolicy::new(read_base_uri),
        None => Ok(IpfsFetchPolicy::default()),
    }
}

fn normalize_read_base_uri(read_base_uri: String) -> Result<String, AppDataError> {
    let normalized = read_base_uri.trim().trim_end_matches('/').to_owned();

    if normalized.is_empty() {
        return Err(AppDataError::Transport(
            "ipfs read base uri must not be empty".to_owned(),
        ));
    }

    Ok(normalized)
}
