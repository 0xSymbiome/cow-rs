//! Runtime-neutral DTOs shared by helper and wasm export layers.

use std::collections::BTreeMap;

use cow_sdk_app_data::{AppDataDoc, AppDataInfo, ValidationResult};
use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, SellTokenSource, TypedDataDomain,
    TypedDataField, TypedDataPayload, TypedDataTypes, UnsignedOrder,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::errors::PureError;

/// Order side accepted by the wasm input DTOs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderKindDto {
    /// Sell an exact sell amount.
    Sell,
    /// Buy an exact buy amount.
    Buy,
}

impl From<OrderKindDto> for OrderKind {
    fn from(value: OrderKindDto) -> Self {
        match value {
            OrderKindDto::Sell => Self::Sell,
            OrderKindDto::Buy => Self::Buy,
        }
    }
}

/// Token-balance mode accepted by the wasm input DTOs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenBalanceDto {
    /// ERC-20 balance or allowance path.
    Erc20,
    /// External Balancer Vault balance path.
    External,
    /// Internal Balancer Vault balance path.
    Internal,
}

impl TokenBalanceDto {
    pub(crate) const fn into_sell_source(self) -> SellTokenSource {
        match self {
            Self::Erc20 => SellTokenSource::Erc20,
            Self::External => SellTokenSource::External,
            Self::Internal => SellTokenSource::Internal,
        }
    }

    pub(crate) fn into_buy_destination(self) -> Result<BuyTokenDestination, PureError> {
        match self {
            Self::Erc20 => Ok(BuyTokenDestination::Erc20),
            Self::Internal => Ok(BuyTokenDestination::Internal),
            Self::External => Err(PureError::unknown_enum("buyTokenBalance", "external")),
        }
    }
}

/// Host-safe order input shared by wasm exports and host smoke tests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderInput {
    /// Sell token address.
    pub sell_token: String,
    /// Buy token address.
    pub buy_token: String,
    /// Optional receiver address. Defaults to the owner at higher layers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Sell amount as a decimal string.
    pub sell_amount: String,
    /// Buy amount as a decimal string.
    pub buy_amount: String,
    /// Expiry timestamp encoded as `uint32`.
    pub valid_to: u32,
    /// App-data hash.
    pub app_data: String,
    /// Fee amount as a decimal string.
    pub fee_amount: String,
    /// Order side.
    pub kind: OrderKindDto,
    /// Whether the order can be partially filled.
    pub partially_fillable: bool,
    /// Sell balance source.
    pub sell_token_balance: TokenBalanceDto,
    /// Buy balance destination.
    pub buy_token_balance: TokenBalanceDto,
}

impl OrderInput {
    /// Parses this DTO into the shared Rust unsigned-order model.
    ///
    /// # Errors
    ///
    /// Returns [`PureError`] when any typed field fails validation.
    pub fn to_unsigned_order(&self) -> Result<UnsignedOrder, PureError> {
        let sell_token = parse_address("sellToken", &self.sell_token)?;
        let buy_token = parse_address("buyToken", &self.buy_token)?;
        let receiver = match &self.receiver {
            Some(receiver) => parse_address("receiver", receiver)?,
            None => Address::new("0x0000000000000000000000000000000000000000")
                .map_err(|error| PureError::invalid("receiver", error.to_string()))?,
        };
        let sell_amount = parse_amount("sellAmount", &self.sell_amount)?;
        let buy_amount = parse_amount("buyAmount", &self.buy_amount)?;
        let app_data = AppDataHash::new(self.app_data.clone())
            .map_err(|error| PureError::invalid("appData", error.to_string()))?;
        let fee_amount = parse_amount("feeAmount", &self.fee_amount)?;
        let buy_token_balance = self.buy_token_balance.into_buy_destination()?;

        Ok(UnsignedOrder::new(
            sell_token,
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            self.valid_to,
            app_data,
            fee_amount,
            self.kind.into(),
            self.partially_fillable,
            self.sell_token_balance.into_sell_source(),
            buy_token_balance,
        ))
    }
}

/// Host-safe app-data document input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDataDocInput {
    /// Application code written into the document.
    pub app_code: String,
    /// Metadata object written into the document.
    pub metadata: Value,
    /// Schema version string.
    pub version: String,
    /// Optional environment label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

impl AppDataDocInput {
    /// Builds an app-data document.
    ///
    /// # Errors
    ///
    /// Returns [`PureError`] when `metadata` is not a JSON object.
    pub fn into_document(self) -> Result<AppDataDoc, PureError> {
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

/// Deployment addresses for the core `CoW` Protocol contracts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentAddresses {
    /// Settlement contract.
    pub settlement: String,
    /// Vault relayer contract.
    pub vault_relayer: String,
    /// `EthFlow` contract.
    pub eth_flow: String,
}

/// Generated order UID and digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedOrderUidDto {
    /// Compact order UID.
    pub order_uid: String,
    /// Underlying order digest.
    pub order_digest: String,
}

/// Host-safe typed-data domain DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedDataDomainDto {
    /// Domain name.
    pub name: String,
    /// Domain version.
    pub version: String,
    /// Numeric chain id.
    pub chain_id: u64,
    /// Verifying contract address.
    pub verifying_contract: String,
}

impl From<&TypedDataDomain> for TypedDataDomainDto {
    fn from(value: &TypedDataDomain) -> Self {
        // The cow `TypedDataDomain` is aliased onto
        // `alloy_sol_types::Eip712Domain` per ADR 0052; all four fields
        // are wrapped in `Option` at the alloy shape but cow construction
        // always sets every one of them. Unwrap with descriptive
        // fallbacks so a partial domain on the wire surfaces as a typed
        // host-DTO with empty / zero defaults rather than panicking.
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

/// Host-safe typed-data field DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Host-safe typed-data envelope with parsed JSON message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Converts a shared typed-data payload to a host-safe DTO.
    ///
    /// # Errors
    ///
    /// Returns [`PureError`] when the canonical JSON message cannot be parsed.
    pub fn from_payload(payload: &TypedDataPayload) -> Result<Self, PureError> {
        Ok(Self {
            domain: TypedDataDomainDto::from(&payload.domain),
            primary_type: payload.primary_type.clone(),
            types: convert_types(&payload.types),
            message: serde_json::from_str(payload.message_json())
                .map_err(|error| PureError::invalid("message", error.to_string()))?,
        })
    }
}

/// App-data result DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDataInfoDto {
    /// CID representation.
    pub cid: String,
    /// Deterministic document content.
    pub app_data_content: String,
    /// App-data hash.
    pub app_data_hex: String,
}

impl From<AppDataInfo> for AppDataInfoDto {
    fn from(value: AppDataInfo) -> Self {
        Self {
            cid: value.cid,
            app_data_content: value.app_data_content,
            app_data_hex: value.app_data_hex,
        }
    }
}

/// App-data validation result DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResultDto {
    /// Whether validation succeeded.
    pub success: bool,
    /// Validation errors when validation failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<String>,
}

impl From<ValidationResult> for ValidationResultDto {
    fn from(value: ValidationResult) -> Self {
        Self {
            success: value.success,
            errors: value.errors.map(|errors| errors.to_string()),
        }
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

pub(crate) fn parse_amount(field: &str, value: &str) -> Result<Amount, PureError> {
    Amount::new(value).map_err(|error| PureError::invalid(field, error.to_string()))
}

fn convert_types(types: &TypedDataTypes) -> BTreeMap<String, Vec<TypedDataFieldDto>> {
    types
        .iter()
        .map(|(name, fields)| {
            (
                name.clone(),
                fields.iter().map(TypedDataFieldDto::from).collect(),
            )
        })
        .collect()
}
