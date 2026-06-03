#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]

//! `CoW` Protocol app-data generation, schema validation, CID conversion, and
//! the IPFS read transport seam.
//!
//! # Quick start
//!
//! Build a minimal SDK-attribution document tagged with a validated
//! [`AppCode`](cow_sdk_core::AppCode), validate it against the bundled
//! JSON schema, and produce a payload ready for `PUT /api/v1/app_data/{hash}`:
//!
//! ```
//! use cow_sdk_core::AppCode;
//! use cow_sdk_app_data::AppDataParams;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let code = AppCode::new("my-app")?;
//! let validated = AppDataParams::new(code).into_validated()?;
//!
//! // validated.info.app_data_content — canonical JSON for PUT /app_data
//! // validated.info.app_data_hex     — 0x-prefixed keccak256 digest
//! # Ok(())
//! # }
//! ```
//!
//! Chain `with_*` setters before the terminal call to add environment,
//! signer, hooks, flashloan hints, or open-ended metadata. See
//! [`AppDataParams`] for the full setter surface and additional examples.

#![warn(missing_docs)]

/// CID conversion helpers for app-data hashes and documents.
pub mod cid;
/// App-data crate error types.
pub mod errors;
/// IPFS fetch policies and read transport seams.
pub mod fetch;
/// Deterministic app-data rendering and digest helpers.
pub mod info;
/// Typed sub-metadata shapes carried inside the app-data envelope.
pub mod metadata;
/// Schema generation and validation helpers.
pub mod schema;
/// Shared app-data types, constants, and configuration structs.
pub mod types;

pub use cid::{app_data_hex_to_cid, cid_to_app_data_hex};
pub use errors::AppDataError;
pub use fetch::{
    IpfsFetchPolicy, IpfsFetchTransport, fetch_doc_from_app_data_hex,
    fetch_doc_from_app_data_hex_with_policy, fetch_doc_from_cid, fetch_doc_from_cid_with_policy,
};
pub use info::{
    APP_DATA_APPROACHING_LIMIT_RATIO, APP_DATA_MAX_BYTES, AppDataSource, AppDataValidated,
    AppDataValidation, AppDataWarning, digest_from_cid, get_app_data_cid, get_app_data_content,
    get_app_data_info, get_app_data_info_hex, stringify_deterministic,
};
pub use metadata::{FlashloanHints, Hook, HookList, QuoteMetadata};
pub use schema::{
    extract_schema_version, generate_app_data_doc, get_app_data_schema, validate_app_data_doc,
};
pub use types::{
    AppDataDoc, AppDataInfo, AppDataParams, DEFAULT_APP_CODE, DEFAULT_IPFS_READ_URI, IpfsConfig,
    LATEST_APP_DATA_VERSION, LATEST_HOOKS_METADATA_VERSION, LATEST_ORDER_CLASS_METADATA_VERSION,
    LATEST_PARTNER_FEE_METADATA_VERSION, LATEST_QUOTE_METADATA_VERSION,
    LATEST_REFERRER_METADATA_VERSION, LATEST_REPLACED_ORDER_METADATA_VERSION,
    LATEST_SCHEMA_VERSION, LATEST_SIGNER_METADATA_VERSION, LATEST_USER_CONSENTS_METADATA_VERSION,
    LATEST_UTM_METADATA_VERSION, LATEST_WIDGET_METADATA_VERSION, LATEST_WRAPPERS_METADATA_VERSION,
    MetadataMap, PartnerFee, PartnerFeePolicy, SchemaVersion, ValidationResult,
};
