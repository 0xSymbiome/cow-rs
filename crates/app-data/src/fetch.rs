use serde_json::Value;

use crate::{AppDataDoc, AppDataError, DEFAULT_IPFS_READ_URI, IpfsConfig, app_data_hex_to_cid};

/// Read transport seam for fetching app-data JSON from IPFS.
pub trait IpfsFetchTransport {
    /// Performs a GET request against `uri`.
    ///
    /// # Errors
    ///
    /// Returns the transport-specific error when the read request fails.
    fn get(&self, uri: &str) -> Result<String, AppDataError>;
}

/// Fetch policy for IPFS reads.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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
    /// Creates a fetch policy with an explicit read base URI.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::Transport`] if the URI is empty after trimming.
    pub fn new(read_base_uri: impl Into<String>) -> Result<Self, AppDataError> {
        let read_base_uri = read_base_uri.into();
        Ok(Self {
            read_base_uri: normalize_read_base_uri(&read_base_uri)?,
        })
    }

    /// Creates a fetch policy from [`IpfsConfig`].
    ///
    /// `read_uri` takes precedence over the general `uri` field.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::Transport`] if the resolved URI is empty after trimming.
    pub fn from_config(config: &IpfsConfig) -> Result<Self, AppDataError> {
        let read_base_uri = config
            .read_uri
            .as_ref()
            .map(|uri| uri.as_inner().as_str())
            .or_else(|| config.uri.as_ref().map(|uri| uri.as_inner().as_str()))
            .unwrap_or(DEFAULT_IPFS_READ_URI);

        Self::new(read_base_uri)
    }

    /// Returns the normalized IPFS read base URI.
    #[must_use]
    pub fn read_base_uri(&self) -> &str {
        &self.read_base_uri
    }

    /// Returns a copy of this policy with a new read base URI.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::Transport`] if the URI is empty after trimming.
    pub fn with_read_base_uri(
        mut self,
        read_base_uri: impl Into<String>,
    ) -> Result<Self, AppDataError> {
        let read_base_uri = read_base_uri.into();
        self.read_base_uri = normalize_read_base_uri(&read_base_uri)?;
        Ok(self)
    }
}

/// Fetches an app-data document by CID using an optional base URI override.
///
/// # Errors
///
/// Returns [`AppDataError`] if the policy is invalid, the transport fails, or
/// the fetched payload is not valid JSON.
pub fn fetch_doc_from_cid(
    cid: &str,
    transport: &impl IpfsFetchTransport,
    ipfs_uri: Option<&str>,
) -> Result<AppDataDoc, AppDataError> {
    fetch_doc_from_cid_with_policy(cid, transport, &policy_from_optional_uri(ipfs_uri)?)
}

/// Fetches an app-data document by CID using an explicit fetch policy.
///
/// # Errors
///
/// Returns [`AppDataError`] if the transport fails or the fetched payload is not valid JSON.
pub fn fetch_doc_from_cid_with_policy(
    cid: &str,
    transport: &impl IpfsFetchTransport,
    policy: &IpfsFetchPolicy,
) -> Result<AppDataDoc, AppDataError> {
    let raw = transport.get(&format!("{}/{}", policy.read_base_uri(), cid))?;
    serde_json::from_str::<Value>(&raw).map_err(AppDataError::from)
}

/// Fetches an app-data document using the app-data hex digest.
///
/// # Errors
///
/// Returns [`AppDataError`] if CID derivation, policy creation, transport execution,
/// or JSON decoding fails.
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

/// Fetches an app-data document using the app-data hex digest and an explicit policy.
///
/// # Errors
///
/// Returns [`AppDataError`] if CID derivation, transport execution, or JSON decoding fails.
pub fn fetch_doc_from_app_data_hex_with_policy(
    app_data_hex: &str,
    transport: &impl IpfsFetchTransport,
    policy: &IpfsFetchPolicy,
) -> Result<AppDataDoc, AppDataError> {
    let cid = app_data_hex_to_cid(app_data_hex).map_err(|err| AppDataError::Transport {
        class: cow_sdk_core::TransportErrorClass::Decode,
        detail: format!("error decoding appDataHex={app_data_hex}: {err}"),
    })?;
    fetch_doc_from_cid_with_policy(&cid, transport, policy)
}

fn policy_from_optional_uri(ipfs_uri: Option<&str>) -> Result<IpfsFetchPolicy, AppDataError> {
    ipfs_uri.map_or_else(|| Ok(IpfsFetchPolicy::default()), IpfsFetchPolicy::new)
}

fn normalize_read_base_uri(read_base_uri: &str) -> Result<String, AppDataError> {
    normalize_ipfs_base_uri("read", read_base_uri)
}

pub(crate) fn normalize_ipfs_base_uri(
    field: &'static str,
    value: &str,
) -> Result<String, AppDataError> {
    let normalized = value.trim().trim_end_matches('/').to_owned();

    if normalized.is_empty() {
        return Err(AppDataError::Transport {
            class: cow_sdk_core::TransportErrorClass::Builder,
            detail: format!("ipfs {field} base uri must not be empty"),
        });
    }

    Ok(normalized)
}
