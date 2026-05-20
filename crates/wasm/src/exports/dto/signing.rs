use std::collections::BTreeMap;

use cow_sdk_core::{TypedDataDomain, TypedDataField, TypedDataPayload};
use cow_sdk_pure_helpers as pure;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tsify::Tsify;
use wasm_bindgen::{JsValue, prelude::*};

use super::{OrderInput, to_js_value};
use crate::exports::errors::WasmError;

/// Generated order UID output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedOrderUidDto {
    /// Compact order UID.
    #[serde(rename = "orderUid")]
    pub order_uid: String,
    /// Underlying order digest.
    pub order_digest: String,
}

impl From<pure::dto::GeneratedOrderUidDto> for GeneratedOrderUidDto {
    fn from(value: pure::dto::GeneratedOrderUidDto) -> Self {
        Self {
            order_uid: value.order_uid,
            order_digest: value.order_digest,
        }
    }
}

/// Typed-data domain DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct TypedDataDomainDto {
    /// Domain name.
    pub name: String,
    /// Domain version.
    pub version: String,
    /// Chain id.
    pub chain_id: u64,
    /// Verifying contract.
    pub verifying_contract: String,
}

impl From<&TypedDataDomain> for TypedDataDomainDto {
    fn from(value: &TypedDataDomain) -> Self {
        // The cow `TypedDataDomain` aliases onto `alloy_sol_types::Eip712Domain`
        // per ADR 0052; every field is wrapped in `Option` at the alloy
        // shape. Cow construction always sets every field; the
        // defensive fallbacks surface a partial domain as a typed DTO
        // (empty strings + zero chainId) rather than panicking inside a
        // `From` impl that cannot return an error.
        let chain_id_u64 = value
            .chain_id
            .and_then(|chain_id| u64::try_from(chain_id).ok())
            .unwrap_or(0);
        let verifying_contract = value
            .verifying_contract
            .map(|address| format!("0x{}", hex::encode(address)))
            .unwrap_or_default();
        Self {
            name: value.name.as_deref().unwrap_or_default().to_owned(),
            version: value.version.as_deref().unwrap_or_default().to_owned(),
            chain_id: chain_id_u64,
            verifying_contract,
        }
    }
}

/// Typed-data field DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TypedDataFieldDto {
    /// Field name.
    pub name: String,
    /// Solidity field type.
    #[serde(rename = "type")]
    pub kind: String,
}

impl From<&TypedDataField> for TypedDataFieldDto {
    fn from(value: &TypedDataField) -> Self {
        Self {
            name: value.name.clone(),
            kind: value.kind.clone(),
        }
    }
}

/// Typed-data envelope DTO.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct TypedDataEnvelopeDto {
    /// Domain metadata.
    pub domain: TypedDataDomainDto,
    /// Primary type.
    pub primary_type: String,
    /// Type map.
    pub types: BTreeMap<String, Vec<TypedDataFieldDto>>,
    /// Parsed message body.
    pub message: Value,
}

impl TypedDataEnvelopeDto {
    /// Builds a DTO from the shared typed-data payload.
    pub fn from_payload(payload: &TypedDataPayload) -> Result<Self, WasmError> {
        Ok(Self {
            domain: TypedDataDomainDto::from(&payload.domain),
            primary_type: payload.primary_type.clone(),
            types: payload
                .types
                .iter()
                .map(|(name, fields)| {
                    (
                        name.clone(),
                        fields.iter().map(TypedDataFieldDto::from).collect(),
                    )
                })
                .collect(),
            message: serde_json::from_str(payload.message_json())?,
        })
    }

    pub(crate) fn callback_value(&self) -> Result<JsValue, JsValue> {
        let value = serde_json::json!({
            "domain": self.domain,
            "types": self.types,
            "primaryType": self.primary_type,
            "message": self.message,
        });
        to_js_value(&value)
    }
}

/// Signed order DTO returned by wallet callback exports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SignedOrderDto {
    /// Compact order UID.
    #[serde(rename = "orderUid")]
    pub order_uid: String,
    /// Signature payload submitted to the orderbook.
    pub signature: String,
    /// Signing scheme.
    pub signing_scheme: String,
    /// Effective owner submitted as `from`.
    pub from: String,
    /// Underlying order digest.
    pub order_digest: String,
    /// Typed-data envelope used for signing.
    pub typed_data: TypedDataEnvelopeDto,
    /// Optional quote id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
}

/// EIP-1193 request DTO.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Eip1193Request {
    /// Provider method.
    pub method: String,
    /// Provider params.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<Value>>,
}

/// Custom EIP-1271 callback request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct CowEip1271SignRequest {
    /// Original order input.
    pub order: OrderInput,
    /// Typed-data envelope.
    pub typed_data: TypedDataEnvelopeDto,
    /// Owner or smart-account address.
    pub owner: String,
    /// Numeric chain id.
    pub chain_id: u32,
}

/// Signed order-cancellation DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SignedCancellationsInput {
    /// Order UIDs to cancel.
    #[serde(rename = "orderUids")]
    pub order_uids: Vec<String>,
    /// Cancellation signature.
    pub signature: String,
    /// ECDSA signing scheme.
    pub signing_scheme: String,
}

pub fn typed_data_json(payload: &TypedDataEnvelopeDto) -> Value {
    serde_json::json!({
        "domain": payload.domain,
        "types": payload.types,
        "primaryType": payload.primary_type,
        "message": payload.message,
    })
}
