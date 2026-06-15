//! Runtime-neutral DTOs shared by helper and wasm export layers.

use cow_sdk_app_data::{AppDataDoc, AppDataError, AppDataInfo};
use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderData, OrderKind, SellTokenSource,
};
use cow_sdk_signing::GeneratedOrderId;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::helpers::errors::PureError;

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
    pub fn to_unsigned_order(&self) -> Result<OrderData, PureError> {
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

        Ok(OrderData::new(
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

/// Converts generated UID data into canonical string DTO fields.
#[must_use]
pub fn generated_order_uid_dto(generated: &GeneratedOrderId) -> GeneratedOrderUidDto {
    GeneratedOrderUidDto {
        order_uid: generated.order_id.to_hex_string(),
        order_digest: generated.order_digest.to_hex_string(),
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
///
/// JavaScript-facing `{success, errors}` projection of the typed
/// `Result<(), AppDataError>` returned by the SDK validator. The rendered
/// error text names only the offending public field and never the
/// caller-supplied value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResultDto {
    /// Whether validation succeeded.
    pub success: bool,
    /// Validation errors when validation failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<String>,
}

impl From<Result<(), AppDataError>> for ValidationResultDto {
    fn from(value: Result<(), AppDataError>) -> Self {
        match value {
            Ok(()) => Self {
                success: true,
                errors: None,
            },
            Err(error) => Self {
                success: false,
                errors: Some(error.to_string()),
            },
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

#[cfg(test)]
mod tests {
    use cow_sdk_core::OrderData;

    use super::{OrderInput, OrderKindDto, TokenBalanceDto};

    const SELL_TOKEN: &str = "0x1111111111111111111111111111111111111111";
    const BUY_TOKEN: &str = "0x2222222222222222222222222222222222222222";
    const ZERO_RECEIVER: &str = "0x0000000000000000000000000000000000000000";
    const CONCRETE_RECEIVER: &str = "0x3333333333333333333333333333333333333333";
    const APP_DATA: &str = "0x337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df";

    fn order_with_receiver(receiver: Option<&str>) -> OrderData {
        OrderInput {
            sell_token: SELL_TOKEN.to_owned(),
            buy_token: BUY_TOKEN.to_owned(),
            receiver: receiver.map(str::to_owned),
            sell_amount: "1000000000000000000".to_owned(),
            buy_amount: "2000000000000000000".to_owned(),
            valid_to: 1_735_689_600,
            app_data: APP_DATA.to_owned(),
            fee_amount: "0".to_owned(),
            kind: OrderKindDto::Sell,
            partially_fillable: false,
            sell_token_balance: TokenBalanceDto::Erc20,
            buy_token_balance: TokenBalanceDto::Erc20,
        }
        .to_unsigned_order()
        .expect("order input should parse into an unsigned order")
    }

    /// ADR 0061: an omitted receiver and an explicit zero receiver resolve to
    /// the same pay-to-owner sentinel, so they build byte-identical `OrderData`
    /// (and therefore the same EIP-712 struct hash, order UID, and signature).
    #[test]
    fn omitted_receiver_matches_explicit_zero_receiver() {
        assert_eq!(
            order_with_receiver(None),
            order_with_receiver(Some(ZERO_RECEIVER)),
            "omitted and explicit-zero receiver must build identical OrderData"
        );
    }

    /// Guards against a vacuous pass and against a boundary that collapses a
    /// concrete receiver into the owner: a real receiver stays distinct from
    /// the pay-to-owner sentinel.
    #[test]
    fn concrete_receiver_differs_from_pay_to_owner_sentinel() {
        assert_ne!(
            order_with_receiver(None),
            order_with_receiver(Some(CONCRETE_RECEIVER)),
            "a concrete receiver must not collapse to the pay-to-owner sentinel"
        );
    }
}
