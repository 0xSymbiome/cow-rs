use std::{
    collections::{BTreeMap, HashMap},
    time::Duration,
};

use cow_sdk_core::{TypedDataDomain, TypedDataField, TypedDataPayload};
use cow_sdk_pure_helpers::{self as pure, errors::PureError};
use cow_sdk_transport_policy::{
    JitterStrategy, LimiterScope, RequestRateLimiter, RetryPolicy, TransportPolicy,
};
use js_sys::Reflect;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use tsify::Tsify;
use wasm_bindgen::{JsValue, prelude::*};

use crate::exports::errors::WasmError;

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

/// App-data document output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AppDataDocDto {
    /// App-data document.
    pub document: Value,
}

impl From<Value> for AppDataDocDto {
    fn from(value: Value) -> Self {
        Self { document: value }
    }
}

/// App-data info output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AppDataInfoDto {
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
    /// Whether validation succeeded.
    pub success: bool,
    /// Errors when validation failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<String>,
}

impl From<pure::dto::ValidationResultDto> for ValidationResultDto {
    fn from(value: pure::dto::ValidationResultDto) -> Self {
        Self {
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
    /// Optional HTTP status text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_text: Option<String>,
    /// Header map.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Body text.
    #[serde(default)]
    pub body: String,
}

/// Retry-policy override accepted by JS client constructors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct RetryPolicyConfig {
    /// Maximum attempts, including the initial request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<u32>,
    /// Base exponential-backoff delay in milliseconds.
    #[serde(
        default,
        alias = "initialDelayMs",
        skip_serializing_if = "Option::is_none"
    )]
    pub base_delay_ms: Option<u32>,
    /// Maximum exponential-backoff delay in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_delay_ms: Option<u32>,
}

/// Rate-limiter bucket scope accepted by JS client constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum LimiterScopeConfig {
    /// One shared bucket.
    Global,
    /// One bucket per resolved host.
    PerHost,
}

impl From<LimiterScopeConfig> for LimiterScope {
    fn from(value: LimiterScopeConfig) -> Self {
        match value {
            LimiterScopeConfig::Global => Self::Global,
            LimiterScopeConfig::PerHost => Self::PerHost,
        }
    }
}

impl From<LimiterScope> for LimiterScopeConfig {
    fn from(value: LimiterScope) -> Self {
        match value {
            LimiterScope::Global => Self::Global,
            LimiterScope::PerHost => Self::PerHost,
            _ => Self::PerHost,
        }
    }
}

/// Request-rate limiter override accepted by JS client constructors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct RequestRateLimiterConfig {
    /// Request tokens granted per interval. Zero disables limiting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens_per_interval: Option<u32>,
    /// Limiter interval in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval_ms: Option<u32>,
    /// Bucket scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<LimiterScopeConfig>,
}

/// Jitter strategy accepted by JS client constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum JitterStrategyConfig {
    /// No retry jitter.
    None,
    /// Full retry jitter.
    Full,
    /// Equal retry jitter.
    Equal,
    /// Decorrelated retry jitter.
    Decorrelated,
}

impl From<JitterStrategyConfig> for JitterStrategy {
    fn from(value: JitterStrategyConfig) -> Self {
        match value {
            JitterStrategyConfig::None => Self::none(),
            JitterStrategyConfig::Full => Self::full(),
            JitterStrategyConfig::Equal => Self::equal(),
            JitterStrategyConfig::Decorrelated => Self::decorrelated(),
        }
    }
}

impl From<JitterStrategy> for JitterStrategyConfig {
    fn from(value: JitterStrategy) -> Self {
        match value {
            JitterStrategy::None => Self::None,
            JitterStrategy::Full { .. } => Self::Full,
            JitterStrategy::Equal { .. } => Self::Equal,
            JitterStrategy::Decorrelated { .. } => Self::Decorrelated,
            _ => Self::Decorrelated,
        }
    }
}

/// Transport-policy override accepted by JS client constructors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct TransportPolicyConfig {
    /// Retry-policy override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<RetryPolicyConfig>,
    /// Rate-limiter override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_rate_limiter: Option<RequestRateLimiterConfig>,
    /// Retry jitter override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jitter_strategy: Option<JitterStrategyConfig>,
    /// Optional transport user-agent value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// Enables or disables transport tracing integration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tracing_enabled: Option<bool>,
}

impl TransportPolicyConfig {
    /// Applies this JS-facing override to a client-specific default policy.
    pub fn apply_to_policy(self, base: TransportPolicy) -> Result<TransportPolicy, WasmError> {
        let mut policy = base;

        if self.retry_policy.is_some() || self.jitter_strategy.is_some() {
            let retry = apply_retry_config(self.retry_policy, policy.retry(), self.jitter_strategy);
            policy = policy.with_retry(retry);
        }

        if let Some(rate_limit) = self.request_rate_limiter {
            let rate_limiter = rate_limit.apply_to_rate_limiter(policy.rate_limit());
            policy = policy.with_rate_limit(rate_limiter);
        }

        if let Some(user_agent) = self.user_agent {
            let client = policy
                .client_policy()
                .clone()
                .try_with_user_agent(user_agent)
                .map_err(|error| {
                    WasmError::invalid("transportPolicy.userAgent", error.to_string())
                })?;
            policy = policy.with_client_policy(client);
        }

        if let Some(tracing_enabled) = self.tracing_enabled {
            policy = policy.with_tracing_enabled(tracing_enabled);
        }

        Ok(policy)
    }
}

fn apply_retry_config(
    config: Option<RetryPolicyConfig>,
    base: &RetryPolicy,
    jitter_strategy: Option<JitterStrategyConfig>,
) -> RetryPolicy {
    let mut builder = RetryPolicy::builder()
        .max_attempts(base.max_attempts())
        .base_delay(base.base_delay())
        .max_delay(base.max_delay())
        .jitter(base.jitter());

    if let Some(config) = config {
        if let Some(max_attempts) = config.max_attempts {
            builder = builder.max_attempts(max_attempts as usize);
        }
        if let Some(base_delay_ms) = config.base_delay_ms {
            builder = builder.base_delay(Duration::from_millis(u64::from(base_delay_ms)));
        }
        if let Some(max_delay_ms) = config.max_delay_ms {
            builder = builder.max_delay(Duration::from_millis(u64::from(max_delay_ms)));
        }
    }

    if let Some(jitter_strategy) = jitter_strategy {
        builder = builder.jitter(jitter_strategy.into());
    }

    builder.build()
}

impl RequestRateLimiterConfig {
    fn apply_to_rate_limiter(self, base: &RequestRateLimiter) -> RequestRateLimiter {
        let mut builder = RequestRateLimiter::builder()
            .tokens_per_interval(base.tokens_per_interval())
            .interval(base.interval())
            .interval_label(base.interval_label())
            .scope(base.scope());

        if let Some(tokens_per_interval) = self.tokens_per_interval {
            builder = builder.tokens_per_interval(tokens_per_interval);
        }
        if let Some(interval_ms) = self.interval_ms {
            builder = builder.interval(Duration::from_millis(u64::from(interval_ms)));
        }
        if let Some(scope) = self.scope {
            builder = builder.scope(scope.into());
        }

        builder.build()
    }
}

pub(crate) fn transport_policy_from_config(
    config: &JsValue,
    default_policy: TransportPolicy,
    timeout: Option<Duration>,
) -> Result<TransportPolicy, JsValue> {
    let mut policy = match optional_js_value(config, "transportPolicy")? {
        Some(value) => {
            let config = serde_wasm_bindgen::from_value::<TransportPolicyConfig>(value).map_err(
                |error| WasmError::invalid("transportPolicy", error.to_string()).into_js(),
            )?;
            config.apply_to_policy(default_policy)?
        }
        None => default_policy,
    };

    if let Some(timeout) = timeout {
        let client = policy.client_policy().clone().with_timeout(timeout);
        policy = policy.with_client_policy(client);
    }

    Ok(policy)
}

fn optional_js_value(value: &JsValue, field: &'static str) -> Result<Option<JsValue>, JsValue> {
    let value = Reflect::get(value, &JsValue::from_str(field))
        .map_err(|error| WasmError::invalid(field, js_message(&error)).into_js())?;
    if value.is_undefined() || value.is_null() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

fn js_message(value: &JsValue) -> String {
    Reflect::get(value, &JsValue::from_str("message"))
        .ok()
        .and_then(|message| message.as_string())
        .or_else(|| value.as_string())
        .unwrap_or_else(|| "JavaScript operation failed".to_owned())
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

/// Orderbook quote request input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OrderQuoteRequestInput {
    /// Sell-token address.
    pub sell_token: String,
    /// Buy-token address.
    pub buy_token: String,
    /// Optional explicit receiver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Quote owner.
    pub from: String,
    /// Quote side.
    pub kind: OrderKindDto,
    /// Sell amount before fee for sell quotes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell_amount_before_fee: Option<String>,
    /// Buy amount after fee for buy quotes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_amount_after_fee: Option<String>,
    /// Relative validity duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    /// Absolute UNIX expiry timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    /// Inline app-data payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
    /// App-data hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<String>,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<TokenBalanceDto>,
    /// Buy-token balance destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<TokenBalanceDto>,
    /// Quote-quality mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_quality: Option<String>,
    /// Expected signing scheme.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signing_scheme: Option<String>,
    /// Whether the eventual order is expected to be on-chain.
    #[serde(default)]
    pub onchain_order: bool,
    /// Optional verification gas limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_gas_limit: Option<u64>,
    /// Optional request timeout in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

impl OrderQuoteRequestInput {
    pub(crate) fn into_value(self) -> Result<Value, WasmError> {
        serde_json::to_value(self).map_err(WasmError::from)
    }
}

/// Orderbook order-creation input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OrderCreationInput {
    /// Sell-token address.
    pub sell_token: String,
    /// Buy-token address.
    pub buy_token: String,
    /// Optional receiver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Sell amount.
    pub sell_amount: String,
    /// Buy amount.
    pub buy_amount: String,
    /// Absolute UNIX expiry timestamp.
    pub valid_to: u32,
    /// Inline app-data payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
    /// App-data hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<String>,
    /// Order-level fee amount. The orderbook accepts only zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_amount: Option<String>,
    /// Strict balance-check flag.
    #[serde(default)]
    pub full_balance_check: bool,
    /// Order side.
    pub kind: OrderKindDto,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<TokenBalanceDto>,
    /// Buy-token balance destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<TokenBalanceDto>,
    /// Signature scheme.
    pub signing_scheme: String,
    /// Raw signature.
    pub signature: String,
    /// Effective owner.
    pub from: String,
    /// Optional quote id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
}

impl OrderCreationInput {
    pub(crate) fn into_value(self) -> Result<Value, WasmError> {
        serde_json::to_value(self).map_err(WasmError::from)
    }
}

/// Partner-fee policy input for trading swap parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PartnerFeePolicyInput {
    /// Volume fee in basis points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_bps: Option<u16>,
    /// Surplus fee in basis points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surplus_bps: Option<u16>,
    /// Price-improvement fee in basis points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_improvement_bps: Option<u16>,
    /// Maximum volume fee in basis points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_volume_bps: Option<u16>,
    /// Fee recipient address.
    pub recipient: String,
}

/// Partner-fee input accepted by trading swap parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(untagged)]
pub enum PartnerFeeInput {
    /// Single partner-fee policy.
    Single(PartnerFeePolicyInput),
    /// Ordered partner-fee policies.
    Multiple(Vec<PartnerFeePolicyInput>),
}

/// Trading swap-parameter input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SwapParametersInput {
    /// Order side.
    pub kind: OrderKindDto,
    /// Optional owner override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Sell-token address.
    pub sell_token: String,
    /// Sell-token decimals.
    pub sell_token_decimals: u8,
    /// Buy-token address.
    pub buy_token: String,
    /// Buy-token decimals.
    pub buy_token_decimals: u8,
    /// Amount interpreted according to `kind`.
    pub amount: String,
    /// Optional environment override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<TokenBalanceDto>,
    /// Buy-token balance destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<TokenBalanceDto>,
    /// Optional slippage tolerance in basis points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
    /// Optional receiver override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Optional relative validity duration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    /// Optional absolute UNIX expiry timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    /// Optional partner-fee metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_fee: Option<PartnerFeeInput>,
}

impl SwapParametersInput {
    pub(crate) fn into_value(self) -> Result<Value, WasmError> {
        serde_json::to_value(self).map_err(WasmError::from)
    }
}

/// Explicit raw GraphQL query input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SubgraphQueryInput {
    /// Raw GraphQL document.
    pub query: String,
    /// Optional GraphQL variables.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<Value>,
    /// Optional operation name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
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
