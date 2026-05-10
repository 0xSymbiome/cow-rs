use std::collections::{BTreeMap, HashMap};

use cow_sdk_core::{TypedDataDomain, TypedDataField, TypedDataPayload};
use cow_sdk_pure_helpers::{self as pure, errors::PureError};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::exports::errors::WasmError;

/// Version tag carried by wasm output envelopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum SchemaVersion {
    /// Current schema version.
    V1,
}

/// Versioned output envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct WasmEnvelope<T> {
    /// Schema version.
    pub schema_version: SchemaVersion,
    /// Envelope payload.
    pub value: T,
}

impl<T> WasmEnvelope<T> {
    /// Wraps a payload in a v1 envelope.
    pub const fn v1(value: T) -> Self {
        Self {
            schema_version: SchemaVersion::V1,
            value,
        }
    }
}

/// Order side accepted by wasm order inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum OrderKindDto {
    /// Sell order.
    Sell,
    /// Buy order.
    Buy,
}

impl From<OrderKindDto> for pure::dto::OrderKindDto {
    fn from(value: OrderKindDto) -> Self {
        match value {
            OrderKindDto::Sell => Self::Sell,
            OrderKindDto::Buy => Self::Buy,
        }
    }
}

/// Token-balance mode accepted by wasm order inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum TokenBalanceDto {
    /// ERC-20 balance or allowance path.
    Erc20,
    /// External Balancer Vault balance path.
    External,
    /// Internal Balancer Vault balance path.
    Internal,
}

impl From<TokenBalanceDto> for pure::dto::TokenBalanceDto {
    fn from(value: TokenBalanceDto) -> Self {
        match value {
            TokenBalanceDto::Erc20 => Self::Erc20,
            TokenBalanceDto::External => Self::External,
            TokenBalanceDto::Internal => Self::Internal,
        }
    }
}

/// Order input shared by signing and UID exports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OrderInput {
    /// Sell token address.
    pub sell_token: String,
    /// Buy token address.
    pub buy_token: String,
    /// Optional receiver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Sell amount.
    pub sell_amount: String,
    /// Buy amount.
    pub buy_amount: String,
    /// Valid-to timestamp.
    pub valid_to: u32,
    /// App-data hash.
    pub app_data: String,
    /// Fee amount.
    pub fee_amount: String,
    /// Order side.
    pub kind: OrderKindDto,
    /// Partial fill flag.
    pub partially_fillable: bool,
    /// Sell balance source.
    pub sell_token_balance: TokenBalanceDto,
    /// Buy balance destination.
    pub buy_token_balance: TokenBalanceDto,
}

impl From<OrderInput> for pure::dto::OrderInput {
    fn from(value: OrderInput) -> Self {
        Self {
            sell_token: value.sell_token,
            buy_token: value.buy_token,
            receiver: value.receiver,
            sell_amount: value.sell_amount,
            buy_amount: value.buy_amount,
            valid_to: value.valid_to,
            app_data: value.app_data,
            fee_amount: value.fee_amount,
            kind: value.kind.into(),
            partially_fillable: value.partially_fillable,
            sell_token_balance: value.sell_token_balance.into(),
            buy_token_balance: value.buy_token_balance.into(),
        }
    }
}

impl From<&cow_sdk_core::UnsignedOrder> for OrderInput {
    fn from(value: &cow_sdk_core::UnsignedOrder) -> Self {
        Self {
            sell_token: value.sell_token.as_str().to_owned(),
            buy_token: value.buy_token.as_str().to_owned(),
            receiver: Some(value.receiver.as_str().to_owned()),
            sell_amount: value.sell_amount.to_string(),
            buy_amount: value.buy_amount.to_string(),
            valid_to: value.valid_to,
            app_data: value.app_data.as_str().to_owned(),
            fee_amount: value.fee_amount.to_string(),
            kind: match value.kind {
                cow_sdk_core::OrderKind::Sell => OrderKindDto::Sell,
                cow_sdk_core::OrderKind::Buy => OrderKindDto::Buy,
            },
            partially_fillable: value.partially_fillable,
            sell_token_balance: match value.sell_token_balance {
                cow_sdk_core::SellTokenSource::Erc20 => TokenBalanceDto::Erc20,
                cow_sdk_core::SellTokenSource::External => TokenBalanceDto::External,
                cow_sdk_core::SellTokenSource::Internal => TokenBalanceDto::Internal,
                _ => TokenBalanceDto::Erc20,
            },
            buy_token_balance: match value.buy_token_balance {
                cow_sdk_core::BuyTokenDestination::Erc20 => TokenBalanceDto::Erc20,
                cow_sdk_core::BuyTokenDestination::Internal => TokenBalanceDto::Internal,
                _ => TokenBalanceDto::Erc20,
            },
        }
    }
}

/// App-data document input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AppDataDocInput {
    /// Application code.
    pub app_code: String,
    /// Metadata object.
    pub metadata: Value,
    /// Schema version.
    pub version: String,
    /// Optional environment label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

impl From<AppDataDocInput> for pure::dto::AppDataDocInput {
    fn from(value: AppDataDocInput) -> Self {
        Self {
            app_code: value.app_code,
            metadata: value.metadata,
            version: value.version,
            environment: value.environment,
        }
    }
}

/// Generated order UID output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedOrderUidDto {
    /// Schema version.
    pub schema_version: SchemaVersion,
    /// Compact order UID.
    #[serde(rename = "orderUid")]
    pub order_uid: String,
    /// Underlying order digest.
    pub order_digest: String,
}

impl From<pure::dto::GeneratedOrderUidDto> for GeneratedOrderUidDto {
    fn from(value: pure::dto::GeneratedOrderUidDto) -> Self {
        Self {
            schema_version: SchemaVersion::V1,
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
        Self {
            name: value.name.clone(),
            version: value.version.clone(),
            chain_id: value.chain_id,
            verifying_contract: value.verifying_contract.as_str().to_owned(),
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
    /// Schema version.
    pub schema_version: SchemaVersion,
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
            schema_version: SchemaVersion::V1,
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
    /// Schema version.
    pub schema_version: SchemaVersion,
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

/// App-data document output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AppDataDocDto {
    /// Schema version.
    pub schema_version: SchemaVersion,
    /// App-data document.
    pub document: Value,
}

impl From<Value> for AppDataDocDto {
    fn from(value: Value) -> Self {
        Self {
            schema_version: SchemaVersion::V1,
            document: value,
        }
    }
}

/// App-data info output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AppDataInfoDto {
    /// Schema version.
    pub schema_version: SchemaVersion,
    /// CID representation.
    pub cid: String,
    /// Deterministic app-data content.
    pub app_data_content: String,
    /// App-data hash.
    pub app_data_hex: String,
}

impl From<pure::dto::AppDataInfoDto> for AppDataInfoDto {
    fn from(value: pure::dto::AppDataInfoDto) -> Self {
        Self {
            schema_version: SchemaVersion::V1,
            cid: value.cid,
            app_data_content: value.app_data_content,
            app_data_hex: value.app_data_hex,
        }
    }
}

/// App-data validation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResultDto {
    /// Schema version.
    pub schema_version: SchemaVersion,
    /// Whether validation succeeded.
    pub success: bool,
    /// Errors when validation failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<String>,
}

impl From<pure::dto::ValidationResultDto> for ValidationResultDto {
    fn from(value: pure::dto::ValidationResultDto) -> Self {
        Self {
            schema_version: SchemaVersion::V1,
            success: value.success,
            errors: value.errors,
        }
    }
}

/// Deployment address output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentAddressesDto {
    /// Schema version.
    pub schema_version: SchemaVersion,
    /// Settlement contract.
    pub settlement: String,
    /// Vault relayer contract.
    pub vault_relayer: String,
    /// EthFlow contract.
    pub eth_flow: String,
}

impl From<pure::dto::DeploymentAddresses> for DeploymentAddressesDto {
    fn from(value: pure::dto::DeploymentAddresses) -> Self {
        Self {
            schema_version: SchemaVersion::V1,
            settlement: value.settlement,
            vault_relayer: value.vault_relayer,
            eth_flow: value.eth_flow,
        }
    }
}

/// Fetch request shape for callback transports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CowFetchRequest {
    /// HTTP method.
    pub method: String,
    /// Absolute URL.
    pub url: String,
    /// Header map.
    pub headers: HashMap<String, String>,
    /// Optional body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Optional timeout in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u32>,
}

/// Fetch response shape returned from callback transports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CowFetchResponse {
    /// HTTP status code.
    pub status: u16,
    /// Header map.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Body text.
    pub body: String,
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

/// Transparent JSON input for orderbook quote requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(transparent)]
pub struct OrderQuoteRequestInput {
    /// Raw JSON value.
    pub value: Value,
}

/// Transparent JSON input for orderbook order creations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(transparent)]
pub struct OrderCreationInput {
    /// Raw JSON value.
    pub value: Value,
}

/// Transparent JSON input for trading swap parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(transparent)]
pub struct SwapParametersInput {
    /// Raw JSON value.
    pub value: Value,
}

/// Transparent JSON input for subgraph raw queries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(transparent)]
pub struct SubgraphQueryInput {
    /// Raw JSON value.
    pub value: Value,
}

pub(crate) fn parse_order(input: OrderInput) -> Result<cow_sdk_core::UnsignedOrder, WasmError> {
    let pure: pure::dto::OrderInput = input.into();
    pure.to_unsigned_order().map_err(WasmError::from)
}

pub(crate) fn parse_chain(chain_id: u32) -> Result<cow_sdk_core::SupportedChainId, WasmError> {
    pure::chains::supported_chain(chain_id).map_err(WasmError::from)
}

pub(crate) fn parse_owner(owner: &str) -> Result<cow_sdk_core::Address, WasmError> {
    pure::dto::parse_address("owner", owner).map_err(WasmError::from)
}

pub(crate) fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    value
        .serialize(&serializer)
        .map_err(|error| WasmError::from(error).into_js())
}

pub(crate) fn from_json_value<T: DeserializeOwned>(
    field: &'static str,
    value: Value,
) -> Result<T, JsValue> {
    serde_json::from_value(value)
        .map_err(|error| WasmError::invalid(field, error.to_string()).into_js())
}

pub(crate) fn orderbook_signing_scheme(
    value: &str,
) -> Result<cow_sdk_orderbook::SigningScheme, WasmError> {
    match value {
        "eip712" | "Eip712" | "EIP712" => Ok(cow_sdk_orderbook::SigningScheme::Eip712),
        "ethsign" | "ethSign" | "EthSign" => Ok(cow_sdk_orderbook::SigningScheme::EthSign),
        "eip1271" | "Eip1271" | "EIP1271" => Ok(cow_sdk_orderbook::SigningScheme::Eip1271),
        "presign" | "preSign" | "PreSign" => Ok(cow_sdk_orderbook::SigningScheme::PreSign),
        other => Err(WasmError::from(PureError::unknown_enum(
            "signingScheme",
            other,
        ))),
    }
}

pub(crate) fn ecdsa_signing_scheme(
    value: &str,
) -> Result<cow_sdk_orderbook::EcdsaSigningScheme, WasmError> {
    match value {
        "eip712" | "Eip712" | "EIP712" => Ok(cow_sdk_orderbook::EcdsaSigningScheme::Eip712),
        "ethsign" | "ethSign" | "EthSign" => Ok(cow_sdk_orderbook::EcdsaSigningScheme::EthSign),
        other => Err(WasmError::from(PureError::unknown_enum(
            "signingScheme",
            other,
        ))),
    }
}

pub(crate) fn typed_data_json(payload: &TypedDataEnvelopeDto) -> Value {
    serde_json::json!({
        "domain": payload.domain,
        "types": payload.types,
        "primaryType": payload.primary_type,
        "message": payload.message,
    })
}
