//! App-data document and CID helpers.

use cow_sdk_app_data::{AppDataDoc, AppDataError, AppDataInfo, IpfsFetchTransport};

use crate::helpers::{dto::AppDataDocInput, errors::PureError};

/// Builds an app-data document from the wasm input DTO.
///
/// # Errors
///
/// Returns [`PureError`] when the DTO cannot be represented as an app-data document.
pub fn document_from_input(input: AppDataDocInput) -> Result<AppDataDoc, PureError> {
    input.into_document()
}

/// Returns canonical app-data info for a document.
///
/// # Errors
///
/// Returns [`AppDataError`] when validation, serialization, or CID conversion fails.
pub fn app_data_info(document: &AppDataDoc) -> Result<AppDataInfo, AppDataError> {
    cow_sdk_app_data::app_data_info(document).map(|validated| validated.info)
}

/// Validates an app-data document against the typed metadata contract.
///
/// # Errors
///
/// Returns [`AppDataError`] when the document fails schema-version or typed
/// metadata validation.
pub fn validate_app_data_doc(document: &AppDataDoc) -> Result<(), AppDataError> {
    cow_sdk_app_data::validate_app_data_doc(document)
}

/// Converts an app-data hash to a CID.
///
/// # Errors
///
/// Returns [`AppDataError`] when the hash is invalid.
pub fn app_data_hex_to_cid(app_data_hex: &str) -> Result<String, AppDataError> {
    cow_sdk_app_data::app_data_hex_to_cid(app_data_hex)
}

/// Converts a CID to an app-data hash.
///
/// # Errors
///
/// Returns [`AppDataError`] when the CID is malformed or unsupported.
pub fn cid_to_app_data_hex(cid: &str) -> Result<String, AppDataError> {
    cow_sdk_app_data::cid_to_app_data_hex(cid)
}

/// Fetches an app-data document by CID through the supplied transport.
///
/// # Errors
///
/// Returns [`AppDataError`] when transport, policy, or JSON decoding fails.
pub async fn fetch_doc_from_cid<T>(
    cid: &str,
    transport: &T,
    ipfs_uri: Option<&str>,
) -> Result<AppDataDoc, AppDataError>
where
    T: IpfsFetchTransport,
{
    cow_sdk_app_data::fetch_doc_from_cid(cid, transport, ipfs_uri).await
}
