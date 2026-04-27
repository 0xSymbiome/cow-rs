use serde_json::Value;

use crate::{
    AppDataDoc, AppDataError, DEFAULT_IPFS_WRITE_URI, IpfsConfig, TransportResponse,
    stringify_deterministic,
};
use cow_sdk_core::Redacted;

/// Upload transport seam for JSON pinning backends.
pub trait IpfsUploadTransport {
    /// Sends a JSON body plus typed-redacted headers to the supplied URI.
    ///
    /// # Errors
    ///
    /// Returns the transport-specific error when the upload request fails.
    fn post_json(
        &self,
        uri: &str,
        body: &str,
        headers: &[(String, Redacted<String>)],
    ) -> Result<TransportResponse, AppDataError>;
}

/// Pins a JSON document through the Pinata `pinJSONToIPFS` API.
///
/// # Errors
///
/// Returns [`AppDataError`] if credentials are missing, request serialization fails,
/// the transport fails, or the response reports an upload error.
pub fn pin_json_in_pinata_ipfs(
    file: &AppDataDoc,
    transport: &impl IpfsUploadTransport,
    ipfs_config: &IpfsConfig,
) -> Result<Value, AppDataError> {
    let pinata_api_key = ipfs_config
        .pinata_api_key
        .clone()
        .filter(|value| !value.as_inner().is_empty())
        .ok_or(AppDataError::MissingIpfsCredentials)?;
    let pinata_api_secret = ipfs_config
        .pinata_api_secret
        .clone()
        .filter(|value| !value.as_inner().is_empty())
        .ok_or(AppDataError::MissingIpfsCredentials)?;
    let write_uri = crate::fetch::normalize_ipfs_base_uri(
        "write",
        ipfs_config
            .write_uri
            .as_ref()
            .map_or(DEFAULT_IPFS_WRITE_URI, |uri| uri.as_inner().as_str()),
    )?;

    let payload = serde_json::json!({
        "pinataContent": file,
        "pinataMetadata": { "name": "appData" },
    });
    let body = stringify_deterministic(&payload)?;
    let headers = vec![
        (
            "Content-Type".to_string(),
            Redacted::new("application/json".to_string()),
        ),
        ("pinata_api_key".to_string(), pinata_api_key),
        ("pinata_secret_api_key".to_string(), pinata_api_secret),
    ];

    let response = transport.post_json(
        &format!("{write_uri}/pinning/pinJSONToIPFS"),
        &body,
        &headers,
    )?;
    let payload: Value = serde_json::from_str(&response.body).map_err(AppDataError::from)?;

    if response.status != 200 {
        let details = payload
            .get("error")
            .and_then(|error| error.get("details").or(Some(error)))
            .and_then(Value::as_str)
            .unwrap_or("IPFS upload failed");
        return Err(AppDataError::Pinning {
            status: Some(response.status),
            message: details.to_string(),
        });
    }

    Ok(payload)
}
