use serde_json::Value;

use crate::{
    AppDataDoc, AppDataError, DEFAULT_IPFS_WRITE_URI, IpfsConfig, IpfsUploadResult,
    TransportResponse, cid_to_app_data_hex, stringify_deterministic,
};

/// Upload transport seam for JSON pinning backends.
pub trait IpfsUploadTransport {
    /// Sends a JSON body plus headers to the supplied URI.
    fn post_json(
        &self,
        uri: &str,
        body: &str,
        headers: &[(String, String)],
    ) -> Result<TransportResponse, AppDataError>;
}

/// Uploads an app-data document using the legacy Pinata flow.
///
/// # Errors
///
/// Returns [`AppDataError`] if credentials are missing, the transport fails, or
/// the response does not contain a valid `IpfsHash`.
pub fn upload_metadata_doc_to_ipfs_legacy(
    app_data_doc: &AppDataDoc,
    transport: &impl IpfsUploadTransport,
    ipfs_config: &IpfsConfig,
) -> Result<IpfsUploadResult, AppDataError> {
    let response = pin_json_in_pinata_ipfs(app_data_doc, transport, ipfs_config)?;
    let cid = response
        .get("IpfsHash")
        .and_then(Value::as_str)
        .ok_or_else(|| AppDataError::Pinning("missing IpfsHash field in response".to_string()))?;

    Ok(IpfsUploadResult {
        app_data: cid_to_app_data_hex(cid)?,
        cid: cid.to_string(),
    })
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
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or(AppDataError::MissingIpfsCredentials)?;
    let pinata_api_secret = ipfs_config
        .pinata_api_secret
        .as_deref()
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
    let payload: Value =
        serde_json::from_str(&response.body).map_err(|err| AppDataError::Json(err.to_string()))?;

    if response.status != 200 {
        let details = payload
            .get("error")
            .and_then(|error| error.get("details").or(Some(error)))
            .and_then(Value::as_str)
            .unwrap_or("IPFS upload failed");
        return Err(AppDataError::Pinning(details.to_string()));
    }

    Ok(payload)
}
