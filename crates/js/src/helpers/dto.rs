//! Runtime-neutral boundary re-exports and lowering helpers for the wasm leaf.
//!
//! Re-exports the boundary shapes call sites name as `helpers::dto::…` — the
//! native order enums from [`cow_sdk_core`] and the leaf's own chain / app-data
//! shapes from [`crate::dto`] — and owns the host-safe lowering helpers
//! (app-data document construction, address and amount parsing) that carry the
//! leaf's typed [`PureError`], so they cannot live in the native crates without
//! dragging that error in.

use cow_sdk_core::{Address, Amount};
use cow_sdk_signing::GeneratedOrderId;
use serde_json::{Map, Value};

pub use cow_sdk_core::{BuyTokenDestination, OrderKind, SellTokenSource};

pub use crate::dto::app_data::{AppDataParams, ValidationResult};
pub use crate::dto::chains::{DeploymentAddresses, GeneratedOrderUid, WrappedNativeToken};

use crate::helpers::errors::PureError;

/// Host-safe lowering of the boundary [`AppDataParams`] into an app-data
/// document.
pub trait AppDataParamsExt {
    /// Builds an app-data document.
    ///
    /// # Errors
    ///
    /// Returns [`PureError`] when `metadata` is not a JSON object.
    fn into_document(self) -> Result<Value, PureError>;
}

impl AppDataParamsExt for AppDataParams {
    fn into_document(self) -> Result<Value, PureError> {
        let Value::Object(metadata) = self.metadata else {
            return Err(PureError::invalid(
                "metadata",
                "metadata must be a JSON object",
            ));
        };

        let mut doc = Map::new();
        doc.insert("appCode".to_owned(), Value::String(self.app_code));
        if let Some(environment) = self.environment {
            doc.insert("environment".to_owned(), Value::String(environment));
        }
        doc.insert("metadata".to_owned(), Value::Object(metadata));
        doc.insert("version".to_owned(), Value::String(self.version));
        Ok(Value::Object(doc))
    }
}

/// Converts generated UID data into canonical string DTO fields.
#[must_use]
pub fn generated_order_uid_dto(generated: &GeneratedOrderId) -> GeneratedOrderUid {
    GeneratedOrderUid {
        order_uid: generated.order_id.to_hex_string(),
        order_digest: generated.order_digest.to_hex_string(),
    }
}

/// Parses an EVM address from a public string field.
///
/// # Errors
///
/// Returns [`PureError`] when the address is malformed.
pub fn parse_address(field: &str, value: &str) -> Result<Address, PureError> {
    Address::new(value).map_err(|error| PureError::invalid(field, error.to_string()))
}

/// Parses a base-10 token amount from a public string field.
///
/// # Errors
///
/// Returns [`PureError`] when the value is not a valid base-10 integer amount.
pub fn parse_amount(field: &str, value: &str) -> Result<Amount, PureError> {
    Amount::new(value).map_err(|error| PureError::invalid(field, error.to_string()))
}
