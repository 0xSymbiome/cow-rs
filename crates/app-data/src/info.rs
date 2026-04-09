use serde_json::Value;
use sha3::{Digest, Keccak256};

use crate::{
    AppDataDoc, AppDataError, AppDataInfo, app_data_hex_to_cid, cid_to_app_data_hex,
    validate_app_data_doc,
};

pub trait AppDataSource {
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

pub fn get_app_data_info_legacy(source: impl AppDataSource) -> Result<AppDataInfo, AppDataError> {
    let (document, app_data_content) = source.into_document_and_content(false)?;
    ensure_valid_document(&document)?;

    let cid = ipfs_cid::generate_cid_v0(app_data_content.as_bytes())
        .map_err(|err| AppDataError::Calculation(err.to_string()))?;
    let app_data_hex = cid_to_app_data_hex(&cid)?;

    Ok(AppDataInfo {
        cid,
        app_data_content,
        app_data_hex,
    })
}

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

pub fn get_app_data_info_hex(source: impl AppDataSource) -> Result<String, AppDataError> {
    Ok(get_app_data_info(source)?.app_data_hex)
}

pub fn get_app_data_cid(source: impl AppDataSource) -> Result<String, AppDataError> {
    Ok(get_app_data_info(source)?.cid)
}

pub fn get_app_data_content(source: impl AppDataSource) -> Result<String, AppDataError> {
    Ok(get_app_data_info(source)?.app_data_content)
}

pub fn digest_from_cid(cid: &str) -> Result<String, AppDataError> {
    cid_to_app_data_hex(cid)
}
