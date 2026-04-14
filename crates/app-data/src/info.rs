use serde_json::Value;
use sha3::{Digest, Keccak256};

use crate::{
    AppDataDoc, AppDataError, AppDataInfo, app_data_hex_to_cid, cid::app_data_bytes_to_legacy_cid,
    cid_to_app_data_hex, validate_app_data_doc,
};

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
            serde_json::to_string(self).map_err(|err| AppDataError::Json(err.to_string()))?
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
            serde_json::to_string(&self).map_err(|err| AppDataError::Json(err.to_string()))?
        };
        Ok((self, content))
    }
}

impl AppDataSource for &str {
    fn into_document_and_content(
        self,
        _deterministic: bool,
    ) -> Result<(AppDataDoc, String), AppDataError> {
        let document: Value =
            serde_json::from_str(self).map_err(|err| AppDataError::Json(err.to_string()))?;
        Ok((document, self.to_string()))
    }
}

impl AppDataSource for String {
    fn into_document_and_content(
        self,
        _deterministic: bool,
    ) -> Result<(AppDataDoc, String), AppDataError> {
        let document: Value =
            serde_json::from_str(&self).map_err(|err| AppDataError::Json(err.to_string()))?;
        Ok((document, self))
    }
}

/// Returns CID, canonical content, and hex digest for the latest app-data path.
///
/// # Errors
///
/// Returns [`AppDataError`] if the source cannot be parsed, validation fails, or
/// CID conversion fails.
pub fn get_app_data_info(source: impl AppDataSource) -> Result<AppDataInfo, AppDataError> {
    let (document, app_data_content) = source.into_document_and_content(true)?;
    ensure_valid_document(&document)?;

    let digest = Keccak256::digest(app_data_content.as_bytes());
    let app_data_hex = format!("0x{}", hex::encode(digest));
    let cid = app_data_hex_to_cid(&app_data_hex)?;

    Ok(AppDataInfo {
        cid,
        app_data_content,
        app_data_hex,
    })
}

/// Returns CID, content, and hex digest for the legacy app-data path.
///
/// # Errors
///
/// Returns [`AppDataError`] if the source cannot be parsed, validation fails, or
/// legacy CID conversion fails.
pub fn get_app_data_info_legacy(source: impl AppDataSource) -> Result<AppDataInfo, AppDataError> {
    let (document, app_data_content) = source.into_document_and_content(false)?;
    ensure_valid_document(&document)?;

    let cid = app_data_bytes_to_legacy_cid(app_data_content.as_bytes())?;
    let app_data_hex = cid_to_app_data_hex(&cid)?;

    Ok(AppDataInfo {
        cid,
        app_data_content,
        app_data_hex,
    })
}

/// Serializes an app-data document with deterministic object-key ordering.
///
/// # Errors
///
/// Returns [`AppDataError::Json`] if any string escaping step fails.
pub fn stringify_deterministic(value: &AppDataDoc) -> Result<String, AppDataError> {
    let mut rendered = String::new();
    write_canonical_json(value, &mut rendered)?;
    Ok(rendered)
}

fn ensure_valid_document(document: &AppDataDoc) -> Result<(), AppDataError> {
    let validation = validate_app_data_doc(document);
    if validation.success {
        return Ok(());
    }

    Err(AppDataError::InvalidAppDataProvided(
        validation
            .errors
            .unwrap_or_else(|| "unknown validation error".to_string()),
    ))
}

fn write_canonical_json(value: &Value, out: &mut String) -> Result<(), AppDataError> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(boolean) => out.push_str(if *boolean { "true" } else { "false" }),
        Value::Number(number) => out.push_str(&number.to_string()),
        Value::String(string) => out.push_str(
            &serde_json::to_string(string).map_err(|err| AppDataError::Json(err.to_string()))?,
        ),
        Value::Array(array) => {
            out.push('[');
            for (index, item) in array.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_canonical_json(item, out)?;
            }
            out.push(']');
        }
        Value::Object(object) => {
            out.push('{');
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            for (index, (key, item)) in entries.into_iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(
                    &serde_json::to_string(key)
                        .map_err(|err| AppDataError::Json(err.to_string()))?,
                );
                out.push(':');
                write_canonical_json(item, out)?;
            }
            out.push('}');
        }
    }

    Ok(())
}

/// Returns only the app-data hex digest for the latest path.
///
/// # Errors
///
/// Returns any error from [`get_app_data_info`].
pub fn get_app_data_info_hex(source: impl AppDataSource) -> Result<String, AppDataError> {
    Ok(get_app_data_info(source)?.app_data_hex)
}

/// Returns only the CID for the latest path.
///
/// # Errors
///
/// Returns any error from [`get_app_data_info`].
pub fn get_app_data_cid(source: impl AppDataSource) -> Result<String, AppDataError> {
    Ok(get_app_data_info(source)?.cid)
}

/// Returns only the serialized app-data content for the latest path.
///
/// # Errors
///
/// Returns any error from [`get_app_data_info`].
pub fn get_app_data_content(source: impl AppDataSource) -> Result<String, AppDataError> {
    Ok(get_app_data_info(source)?.app_data_content)
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
