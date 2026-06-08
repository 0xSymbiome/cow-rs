use async_trait::async_trait;
use serde_json::Value;

use crate::{AppDataDoc, AppDataError, DEFAULT_IPFS_READ_URI, IpfsConfig, app_data_hex_to_cid};

/// Read transport seam for fetching app-data JSON from IPFS.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait IpfsFetchTransport {
    /// Performs a GET request against `uri`.
    ///
    /// # Errors
    ///
    /// Returns the transport-specific error when the read request fails.
    async fn get(&self, uri: &str) -> Result<String, AppDataError>;
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
pub async fn fetch_doc_from_cid(
    cid: &str,
    transport: &impl IpfsFetchTransport,
    ipfs_uri: Option<&str>,
) -> Result<AppDataDoc, AppDataError> {
    fetch_doc_from_cid_with_policy(cid, transport, &policy_from_optional_uri(ipfs_uri)?).await
}

/// Fetches an app-data document by CID using an explicit fetch policy.
///
/// This is the shared IPFS read leaf: every `fetch_doc_*` entry point funnels
/// here, so the single `fetch_doc_from_cid_with_policy` span covers each fetch
/// path exactly once. The span records the requested `cid` and a stable
/// `endpoint` label only; the configured read base URI — which may carry a
/// gateway credential — is never recorded, matching the `Redacted<String>`
/// posture of [`IpfsConfig`].
///
/// # Errors
///
/// Returns [`AppDataError`] if the transport fails or the fetched payload is not valid JSON.
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        skip_all,
        fields(
            endpoint = "app_data.fetch_doc_from_cid",
            cid = %cid,
        ),
    ),
)]
pub async fn fetch_doc_from_cid_with_policy(
    cid: &str,
    transport: &impl IpfsFetchTransport,
    policy: &IpfsFetchPolicy,
) -> Result<AppDataDoc, AppDataError> {
    let raw = transport
        .get(&format!("{}/{}", policy.read_base_uri(), cid))
        .await?;
    serde_json::from_str::<Value>(&raw).map_err(AppDataError::from)
}

/// Fetches an app-data document using the app-data hex digest.
///
/// The primary way to read a document you uploaded is the orderbook
/// `GET /app_data/{hash}` request, which is served from the orderbook database
/// and needs no IPFS gateway. This helper is the secondary, not-in-database
/// path: it derives the keccak-256 `CIDv1` from `app_data_hex` and reads it
/// through the injected transport, so `ipfs_uri` must point at a gateway that
/// can resolve keccak-CID documents — a generic public gateway cannot.
///
/// # Errors
///
/// Returns [`AppDataError`] if CID derivation, policy creation, transport execution,
/// or JSON decoding fails.
pub async fn fetch_doc_from_app_data_hex(
    app_data_hex: &str,
    transport: &impl IpfsFetchTransport,
    ipfs_uri: Option<&str>,
) -> Result<AppDataDoc, AppDataError> {
    fetch_doc_from_app_data_hex_with_policy(
        app_data_hex,
        transport,
        &policy_from_optional_uri(ipfs_uri)?,
    )
    .await
}

/// Fetches an app-data document using the app-data hex digest and an explicit policy.
///
/// # Errors
///
/// Returns [`AppDataError`] if CID derivation, transport execution, or JSON decoding fails.
pub async fn fetch_doc_from_app_data_hex_with_policy(
    app_data_hex: &str,
    transport: &impl IpfsFetchTransport,
    policy: &IpfsFetchPolicy,
) -> Result<AppDataDoc, AppDataError> {
    let cid = app_data_hex_to_cid(app_data_hex).map_err(|err| AppDataError::Transport {
        class: cow_sdk_core::TransportErrorClass::Decode,
        detail: format!("error decoding appDataHex={app_data_hex}: {err}").into(),
    })?;
    fetch_doc_from_cid_with_policy(&cid, transport, policy).await
}

fn policy_from_optional_uri(ipfs_uri: Option<&str>) -> Result<IpfsFetchPolicy, AppDataError> {
    ipfs_uri.map_or_else(|| Ok(IpfsFetchPolicy::default()), IpfsFetchPolicy::new)
}

fn normalize_read_base_uri(read_base_uri: &str) -> Result<String, AppDataError> {
    let normalized = read_base_uri.trim().trim_end_matches('/').to_owned();

    if normalized.is_empty() {
        return Err(AppDataError::Transport {
            class: cow_sdk_core::TransportErrorClass::Builder,
            detail: "ipfs read base uri must not be empty".to_owned().into(),
        });
    }

    Ok(normalized)
}
