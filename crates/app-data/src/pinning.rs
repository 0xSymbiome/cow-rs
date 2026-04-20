use serde_json::Value;

use crate::{
    AppDataDoc, AppDataError, DEFAULT_IPFS_WRITE_URI, IpfsConfig, TransportResponse,
    stringify_deterministic,
};

/// Upload transport seam for JSON pinning backends.
pub trait IpfsUploadTransport {
    /// Sends a JSON body plus headers to the supplied URI.
    ///
    /// # Errors
    ///
    /// Returns the transport-specific error when the upload request fails.
    fn post_json(
        &self,
        uri: &str,
        body: &str,
        headers: &[(String, String)],
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
        .as_ref()
        .map(|value| value.as_inner().as_str())
        .filter(|value| !value.is_empty())
        .ok_or(AppDataError::MissingIpfsCredentials)?;
    let pinata_api_secret = ipfs_config
        .pinata_api_secret
        .as_ref()
        .map(|value| value.as_inner().as_str())
        .filter(|value| !value.is_empty())
        .ok_or(AppDataError::MissingIpfsCredentials)?;
    let write_uri = ipfs_config
        .write_uri
        .as_deref()
        .unwrap_or(DEFAULT_IPFS_WRITE_URI)
        .trim_end_matches('/');

    let payload = serde_json::json!({
        "pinataContent": file,
        "pinataMetadata": { "name": "appData" },
    });
    let body = stringify_deterministic(&payload)?;
    let headers = vec![
        ("Content-Type".to_string(), "application/json".to_string()),
        ("pinata_api_key".to_string(), pinata_api_key.to_string()),
        (
            "pinata_secret_api_key".to_string(),
            pinata_api_secret.to_string(),
        ),
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
