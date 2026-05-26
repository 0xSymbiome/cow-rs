use std::ops::Deref;

use alloy_primitives::keccak256;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    AppDataDoc, AppDataError, AppDataInfo, app_data_hex_to_cid, cid_to_app_data_hex,
    validate_app_data_doc,
};

/// Client-side size ceiling for stringified app-data documents.
///
/// Matches the upstream orderbook's 8 KB app-data limit. Surfaces the limit as
/// a typed [`AppDataError::TooLarge`] at the client boundary instead of
/// waiting for the orderbook's 422 response.
pub const APP_DATA_MAX_BYTES: usize = 8192;

/// Fraction of [`APP_DATA_MAX_BYTES`] at which a typed
/// [`AppDataWarning::ApproachingSizeLimit`] is emitted.
///
/// A stringified deterministic payload whose byte size reaches or exceeds
/// this fraction of the configured ceiling surfaces a soft warning so
/// callers can react before the hard [`AppDataError::TooLarge`] path fires.
pub const APP_DATA_APPROACHING_LIMIT_RATIO: f64 = 0.75;

/// Successful outcome of [`get_app_data_info`], pairing the canonical
/// [`AppDataInfo`] result with typed validation metadata.
///
/// `AppDataValidated` implements [`Deref`] with
/// [`AppDataInfo`] as its target so every existing field access via dot
/// notation (for example `validated.app_data_hex`) continues to compile
/// without code change. Destructure `validated.info` when moving the
/// underlying [`AppDataInfo`] out of the wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AppDataValidated {
    /// Canonical [`AppDataInfo`] for the validated document.
    pub info: AppDataInfo,
    /// Validation metadata captured alongside the canonical result.
    pub validation: AppDataValidation,
}

impl AppDataValidated {
    /// Creates a validated app-data result from canonical info and validation metadata.
    #[must_use]
    pub const fn new(info: AppDataInfo, validation: AppDataValidation) -> Self {
        Self { info, validation }
    }
}

impl Deref for AppDataValidated {
    type Target = AppDataInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

/// Validation metadata captured alongside a successful [`AppDataValidated`]
/// result.
///
/// The struct is `#[non_exhaustive]` so future additions to the validation
/// surface may be introduced as a minor change without breaking downstream
/// exhaustive matches.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppDataValidation {
    /// Byte size of the stringified deterministic payload — the same value
    /// the reviewed services validator measures.
    pub bytes_used: usize,
    /// Soft-warning channel carrying non-fatal observations about the
    /// validated document. Hard errors remain on the
    /// [`AppDataError`] path.
    pub warnings: Vec<AppDataWarning>,
}

/// Non-fatal observation emitted alongside a successful
/// [`AppDataValidated`] result.
///
/// The enum is `#[non_exhaustive]` so future soft-warning variants may be
/// introduced as a minor change without breaking downstream exhaustive
/// matches. Hard errors — unknown keys, schema violations, and oversized
/// payloads — stay on the [`AppDataError`] path and
/// are never demoted to warnings.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AppDataWarning {
    /// The stringified deterministic payload reached the configured
    /// near-limit fraction of [`APP_DATA_MAX_BYTES`]. The payload is still
    /// within the hard ceiling; callers that want headroom for subsequent
    /// edits may want to trim metadata before sealing the document.
    #[serde(rename_all = "camelCase")]
    ApproachingSizeLimit {
        /// Byte size of the stringified deterministic payload.
        bytes_used: usize,
        /// Configured hard ceiling for stringified app-data documents.
        max_bytes: usize,
    },
}

/// Source abstraction for app-data generation helpers.
pub trait AppDataSource {
    /// Converts the source into a parsed document plus the serialized content string.
    ///
    /// When `deterministic` is true, implementations should use canonical key ordering.
    ///
    /// # Errors
    ///
    /// Returns an error when the source cannot be parsed or serialized into a valid app-data
    /// document.
    fn into_document_and_content(
        self,
        deterministic: bool,
    ) -> Result<(AppDataDoc, String), AppDataError>;
}

impl AppDataSource for &AppDataDoc {
    fn into_document_and_content(
        self,
        deterministic: bool,
    ) -> Result<(AppDataDoc, String), AppDataError> {
        let content = if deterministic {
            stringify_deterministic(self)?
        } else {
            serde_json::to_string(self).map_err(AppDataError::from)?
        };
        Ok((self.clone(), content))
    }
}

impl AppDataSource for AppDataDoc {
    fn into_document_and_content(
        self,
        deterministic: bool,
    ) -> Result<(AppDataDoc, String), AppDataError> {
        let content = if deterministic {
            stringify_deterministic(&self)?
        } else {
            serde_json::to_string(&self).map_err(AppDataError::from)?
        };
        Ok((self, content))
    }
}

impl AppDataSource for &str {
    fn into_document_and_content(
        self,
        _deterministic: bool,
    ) -> Result<(AppDataDoc, String), AppDataError> {
        let document: Value = serde_json::from_str(self).map_err(AppDataError::from)?;
        Ok((document, self.to_string()))
    }
}

impl AppDataSource for String {
    fn into_document_and_content(
        self,
        _deterministic: bool,
    ) -> Result<(AppDataDoc, String), AppDataError> {
        let document: Value = serde_json::from_str(&self).map_err(AppDataError::from)?;
        Ok((document, self))
    }
}

/// Returns the canonical [`AppDataInfo`] plus the typed
/// [`AppDataValidation`] metadata for the supplied app-data source.
///
/// On the success path the wrapper carries the deterministic payload size
/// in `validation.bytes_used` and an ordered soft-warning channel in
/// `validation.warnings`. A stringified payload whose byte size reaches or
/// exceeds [`APP_DATA_APPROACHING_LIMIT_RATIO`] of [`APP_DATA_MAX_BYTES`]
/// emits an [`AppDataWarning::ApproachingSizeLimit`]; the hard
/// [`AppDataError::TooLarge`] path continues to fire at the configured
/// ceiling and the wrapper is never constructed on the error path.
///
/// # Errors
///
/// Returns [`AppDataError`] if the source cannot be parsed, validation
/// fails, the stringified payload exceeds [`APP_DATA_MAX_BYTES`], or CID
/// conversion fails.
pub fn get_app_data_info(source: impl AppDataSource) -> Result<AppDataValidated, AppDataError> {
    let (document, app_data_content) = source.into_document_and_content(true)?;
    ensure_document_under_size_limit(&app_data_content, APP_DATA_MAX_BYTES)?;
    ensure_valid_document(&document)?;

    let bytes_used = app_data_content.len();
    let digest = keccak256(app_data_content.as_bytes());
    let app_data_hex = alloy_primitives::hex::encode_prefixed(digest);
    let cid = app_data_hex_to_cid(&app_data_hex)?;

    let info = AppDataInfo {
        cid,
        app_data_content,
        app_data_hex,
    };

    let mut warnings = Vec::new();
    if approaching_size_limit(bytes_used, APP_DATA_MAX_BYTES) {
        warnings.push(AppDataWarning::ApproachingSizeLimit {
            bytes_used,
            max_bytes: APP_DATA_MAX_BYTES,
        });
    }

    Ok(AppDataValidated {
        info,
        validation: AppDataValidation {
            bytes_used,
            warnings,
        },
    })
}

fn approaching_size_limit(bytes_used: usize, max_bytes: usize) -> bool {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "byte-size and ratio multiplication produces values that fit back inside usize with the floor the test contract requires"
    )]
    let threshold = (max_bytes as f64 * APP_DATA_APPROACHING_LIMIT_RATIO) as usize;
    bytes_used >= threshold
}

/// Serializes an app-data document as RFC 8785 canonical JSON via
/// [`serde_jcs::to_string`].
///
/// The output applies UTF-16 code-unit key ordering, decimal-only number
/// serialisation, and the canonical insignificant-whitespace rules
/// specified by RFC 8785 (JSON Canonicalization Scheme). ASCII-only
/// documents serialise byte-identically to the previous bytewise key
/// ordering; documents whose object keys carry characters whose UTF-16
/// representation diverges from their UTF-8 byte ordering now match the
/// canonical RFC 8785 form, closing a latent gap with the upstream
/// `@cowprotocol/cow-sdk` TypeScript implementation.
///
/// # Errors
///
/// Returns [`AppDataError::Json`] if the canonicalisation pass fails.
pub fn stringify_deterministic(value: &AppDataDoc) -> Result<String, AppDataError> {
    serde_jcs::to_string(value).map_err(AppDataError::from)
}

const fn ensure_document_under_size_limit(
    content: &str,
    max_bytes: usize,
) -> Result<(), AppDataError> {
    let actual = content.len();
    if actual > max_bytes {
        return Err(AppDataError::TooLarge {
            actual_bytes: actual,
            max_bytes,
        });
    }
    Ok(())
}

fn ensure_valid_document(document: &AppDataDoc) -> Result<(), AppDataError> {
    let validation = validate_app_data_doc(document);
    if validation.success {
        return Ok(());
    }

    Err(AppDataError::InvalidAppDataProvided {
        field: "document",
        reason: cow_sdk_core::ValidationReason::BadShape {
            details: "document failed the embedded JSON schema validation",
        },
    })
}

/// Returns only the app-data hex digest.
///
/// # Errors
///
/// Returns any error from [`get_app_data_info`].
pub fn get_app_data_info_hex(source: impl AppDataSource) -> Result<String, AppDataError> {
    Ok(get_app_data_info(source)?.info.app_data_hex)
}

/// Returns only the CID derived from the app-data content.
///
/// # Errors
///
/// Returns any error from [`get_app_data_info`].
pub fn get_app_data_cid(source: impl AppDataSource) -> Result<String, AppDataError> {
    Ok(get_app_data_info(source)?.info.cid)
}

/// Returns only the serialized app-data content.
///
/// # Errors
///
/// Returns any error from [`get_app_data_info`].
pub fn get_app_data_content(source: impl AppDataSource) -> Result<String, AppDataError> {
    Ok(get_app_data_info(source)?.info.app_data_content)
}

/// Extracts the app-data hex digest from a supported CID.
///
/// # Errors
///
/// Returns [`AppDataError::InvalidCid`] if the CID is malformed or unsupported.
pub fn digest_from_cid(cid: &str) -> Result<String, AppDataError> {
    cid_to_app_data_hex(cid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    #[test]
    fn stringify_deterministic_orders_keys_without_corrupting_arrays() {
        let document = json!({
            "version": "0.7.0",
            "metadata": {
                "nested": {
                    "b": 2,
                    "a": 1
                },
                "array": [3, 2, 1]
            },
            "appCode": "CoW Swap"
        });

        assert_eq!(
            stringify_deterministic(&document).unwrap(),
            r#"{"appCode":"CoW Swap","metadata":{"array":[3,2,1],"nested":{"a":1,"b":2}},"version":"0.7.0"}"#
        );
    }

    #[test]
    fn string_sources_preserve_the_original_content_and_document_shape() {
        let raw = r#"{"metadata":{"b":2,"a":1},"version":"0.7.0","appCode":"CoW Swap"}"#;
        let expected = serde_json::from_str::<Value>(raw).unwrap();

        let (borrowed_document, borrowed_content) =
            <&str as AppDataSource>::into_document_and_content(raw, false).unwrap();
        assert_eq!(borrowed_document, expected);
        assert_eq!(borrowed_content, raw);

        let owned = raw.to_owned();
        let (owned_document, owned_content) =
            owned.clone().into_document_and_content(false).unwrap();
        assert_eq!(owned_document, expected);
        assert_eq!(owned_content, owned);
    }

    #[test]
    fn accessors_match_the_primary_app_data_info_result() {
        let document = json!({
            "appCode": "CoW Swap",
            "metadata": {
                "quote": {
                    "version": "0.2.0",
                    "slippageBips": "5"
                }
            },
            "version": "0.7.0"
        });

        let info = get_app_data_info(&document).unwrap();

        assert_eq!(get_app_data_info_hex(&document).unwrap(), info.app_data_hex);
        assert_eq!(get_app_data_cid(&document).unwrap(), info.cid);
        assert_eq!(
            get_app_data_content(&document).unwrap(),
            info.app_data_content
        );
        assert_eq!(digest_from_cid(&info.cid).unwrap(), info.app_data_hex);
    }
}
