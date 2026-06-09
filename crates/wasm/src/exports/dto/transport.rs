use std::collections::HashMap;
#[cfg(feature = "transport-policy")]
use std::time::Duration;

#[cfg(feature = "transport-policy")]
use cow_sdk_core::transport::policy::{
    JitterStrategy, LimiterScope, RequestRateLimiter, RetryPolicy, TransportPolicy,
};
#[cfg(feature = "transport-policy")]
use js_sys::Reflect;
use serde::{Deserialize, Serialize};
#[cfg(feature = "transport-policy")]
use tsify::Tsify;
#[cfg(feature = "transport-policy")]
use wasm_bindgen::{JsValue, prelude::*};

#[cfg(feature = "transport-policy")]
use crate::exports::errors::WasmError;

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
#[cfg(feature = "transport-policy")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetryPolicyConfig {
    /// Maximum attempts, including the initial request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<u32>,
    /// Base exponential-backoff delay in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_delay_ms: Option<u32>,
    /// Maximum exponential-backoff delay in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_delay_ms: Option<u32>,
}

/// Rate-limiter bucket scope accepted by JS client constructors.
#[cfg(feature = "transport-policy")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum LimiterScopeConfig {
    /// One shared bucket.
    Global,
    /// One bucket per resolved host.
    PerHost,
}

#[cfg(feature = "transport-policy")]
impl From<LimiterScopeConfig> for LimiterScope {
    fn from(value: LimiterScopeConfig) -> Self {
        match value {
            LimiterScopeConfig::Global => Self::Global,
            LimiterScopeConfig::PerHost => Self::PerHost,
        }
    }
}

#[cfg(feature = "transport-policy")]
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
#[cfg(feature = "transport-policy")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
#[cfg(feature = "transport-policy")]
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

#[cfg(feature = "transport-policy")]
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

#[cfg(feature = "transport-policy")]
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
#[cfg(feature = "transport-policy")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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

#[cfg(feature = "transport-policy")]
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

#[cfg(feature = "transport-policy")]
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

#[cfg(feature = "transport-policy")]
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

#[cfg(feature = "transport-policy")]
pub fn transport_policy_from_config(
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

#[cfg(feature = "transport-policy")]
fn optional_js_value(value: &JsValue, field: &'static str) -> Result<Option<JsValue>, JsValue> {
    let value = Reflect::get(value, &JsValue::from_str(field))
        .map_err(|error| WasmError::invalid(field, js_message(&error)).into_js())?;
    if value.is_undefined() || value.is_null() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

#[cfg(feature = "transport-policy")]
fn js_message(value: &JsValue) -> String {
    Reflect::get(value, &JsValue::from_str("message"))
        .ok()
        .and_then(|message| message.as_string())
        .or_else(|| value.as_string())
        .unwrap_or_else(|| "JavaScript operation failed".to_owned())
}
