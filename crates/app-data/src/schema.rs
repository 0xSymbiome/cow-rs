use serde_json::Value;

use cow_sdk_core::{AppCode, ValidationReason};

use crate::{
    AppDataDoc, AppDataError, AppDataParams, DEFAULT_APP_CODE, LATEST_APP_DATA_VERSION, PartnerFee,
    QuoteMetadata, SchemaVersion, metadata::FlashloanHints,
};

/// Builds a canonical app-data document from typed parameters.
///
/// Most callers should prefer the fluent terminal
/// [`AppDataParams::into_doc`] which chains naturally with the `.with_*`
/// setters:
///
/// ```
/// use cow_sdk_core::AppCode;
/// use cow_sdk_app_data::AppDataParams;
///
/// # fn main() -> Result<(), cow_sdk_core::AppCodeError> {
/// let code = AppCode::new("my-app")?;
/// let doc = AppDataParams::new(code)
///     .with_environment("production")
///     .into_doc();
/// # Ok(())
/// # }
/// ```
///
/// This free-function form is retained for composed flows such as the
/// typed merge pipeline and for callers building params through
/// [`AppDataParams::default`] + reflective mutation.
///
/// The typed `signer` and `flashloan` sub-fields on [`AppDataParams`] are
/// merged into the nested `metadata` object in their reviewed camelCase
/// positions before the document is sealed, so the document carries the
/// same wire shape whether the caller supplied the typed fields directly
/// or folded them through the open-ended metadata map.
///
/// # Panics
///
/// Panics only if the typed `flashloan` sub-field ever stops serializing to
/// JSON — which cannot happen for values produced through the public
/// constructors.
#[must_use]
pub fn generate_app_data_doc(params: AppDataParams) -> AppDataDoc {
    // SAFETY: typed metadata values are SDK-owned serde shapes; serialization
    // failure means a broken crate invariant rather than caller input.
    let metadata = params
        .metadata_wire_value()
        .expect("typed flashloan metadata must remain serializable");
    let mut doc = serde_json::Map::new();
    doc.insert(
        "appCode".to_string(),
        Value::String(
            params
                .app_code
                .map_or_else(|| DEFAULT_APP_CODE.to_string(), AppCode::into_inner),
        ),
    );
    if let Some(environment) = params.environment {
        doc.insert("environment".to_string(), Value::String(environment));
    }
    doc.insert("metadata".to_string(), metadata);
    doc.insert(
        "version".to_string(),
        Value::String(LATEST_APP_DATA_VERSION.to_string()),
    );
    Value::Object(doc)
}

/// Validates an app-data document against the typed metadata contract.
///
/// The document must carry a string `version` field in
/// `<major>.<minor>.<patch>` form. Every metadata family the SDK models —
/// `flashloan`, `partnerFee`, and `quote` — is bound-checked when present in
/// its current wire shape. Metadata the SDK does not model, and values carried
/// in an earlier wire shape that no longer parses into the current typed form,
/// are left untouched so the result is never stricter than the orderbook's own
/// acceptance contract.
///
/// This is the same single validation pass [`crate::app_data_info`] runs
/// before deriving the document digest, so callers that need both the check
/// and the derived identifiers should call [`crate::app_data_info`] once
/// rather than validating separately first.
///
/// # Errors
///
/// Every error rendering names only the offending public field and never the
/// caller-supplied value:
///
/// - [`AppDataError::MissingSchemaVersion`] when the document does not
///   contain a string-valued `version` field.
/// - [`AppDataError::InvalidSchemaVersion`] when `version` is not in
///   `<major>.<minor>.<patch>` form.
/// - [`AppDataError::InvalidAppDataProvided`] when a present
///   `metadata.flashloan` or `metadata.partnerFee` value does not match its
///   typed wire shape, or when a typed `metadata.quote` carries an
///   out-of-range `slippageBips`.
/// - [`AppDataError::InvalidFlashloanHints`] when a well-shaped
///   `metadata.flashloan` fails its bound checks.
/// - [`AppDataError::InvalidPartnerFee`] when a well-shaped
///   `metadata.partnerFee` fails its bound checks.
pub fn validate_app_data_doc(app_data_doc: &AppDataDoc) -> Result<(), AppDataError> {
    let version = extract_schema_version(app_data_doc)?;
    SchemaVersion::new(version)?;

    let Some(metadata) = app_data_doc.get("metadata").and_then(Value::as_object) else {
        return Ok(());
    };

    // Families the reviewed services parser also models are validated
    // strictly: a present-but-malformed value is rejected with a safe,
    // family-named error rather than the caller-supplied bytes.
    if let Some(value) = metadata.get("flashloan") {
        serde_json::from_value::<FlashloanHints>(value.clone())
            .map_err(|_| AppDataError::InvalidAppDataProvided {
                field: "metadata.flashloan",
                reason: ValidationReason::BadShape {
                    details: "value does not match the typed flash-loan hint shape",
                },
            })?
            .validate()?;
    }
    if let Some(value) = metadata.get("partnerFee") {
        PartnerFee::from_value(value.clone())
            .map_err(|_| AppDataError::InvalidAppDataProvided {
                field: "metadata.partnerFee",
                reason: ValidationReason::BadShape {
                    details: "value does not match a supported partner-fee policy shape",
                },
            })?
            .validate()?;
    }

    // `quote` is bound-checked opportunistically: earlier wire shapes carried
    // by historical documents no longer parse into the current typed quote and
    // are passed through unchanged so they continue to hash.
    if let Some(value) = metadata.get("quote")
        && let Ok(quote) = QuoteMetadata::from_value(value.clone())
    {
        quote.validate()?;
    }

    Ok(())
}

/// Extracts the schema version string from an app-data document.
///
/// # Errors
///
/// Returns [`AppDataError::MissingSchemaVersion`] when the document does not
/// contain a string-valued `version` field.
pub fn extract_schema_version(app_data_doc: &AppDataDoc) -> Result<&str, AppDataError> {
    app_data_doc
        .get("version")
        .and_then(Value::as_str)
        .ok_or(AppDataError::MissingSchemaVersion)
}
