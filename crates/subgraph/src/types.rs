//! Public request and response DTOs for subgraph queries.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Explicit raw GraphQL request input for [`crate::SubgraphApi::run_query`].
///
/// This request shape keeps the document, variables, and optional operation
/// name visible to callers instead of inferring them from the GraphQL string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubgraphQueryRequest {
    /// Raw GraphQL document sent to the subgraph endpoint.
    pub document: String,
    /// Optional variables object sent alongside `document`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<Value>,
    /// Optional operation name for multi-operation or explicitly named queries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
}

impl SubgraphQueryRequest {
    /// Creates a request from a raw GraphQL document.
    #[must_use]
    pub fn new(document: impl Into<String>) -> Self {
        Self {
            document: document.into(),
            variables: None,
            operation_name: None,
        }
    }

    /// Returns the raw GraphQL document.
    #[must_use]
    pub fn document(&self) -> &str {
        &self.document
    }

    /// Returns the request variables, if present.
    #[must_use]
    pub fn variables(&self) -> Option<&Value> {
        self.variables.as_ref()
    }

    /// Returns the request operation name, if present.
    #[must_use]
    pub fn operation_name(&self) -> Option<&str> {
        self.operation_name.as_deref()
    }

    /// Returns a copy of this request with explicit variables.
    #[must_use]
    pub fn with_variables(mut self, variables: Value) -> Self {
        self.variables = Some(variables);
        self
    }

    /// Returns a copy of this request with optional variables.
    #[must_use]
    pub fn with_optional_variables(mut self, variables: Option<Value>) -> Self {
        self.variables = variables;
        self
    }

    /// Returns a copy of this request with an explicit operation name.
    #[must_use]
    pub fn with_operation_name(mut self, operation_name: impl Into<String>) -> Self {
        self.operation_name = Some(operation_name.into());
        self
    }
}

impl From<&str> for SubgraphQueryRequest {
    fn from(document: &str) -> Self {
        Self::new(document)
    }
}

impl From<String> for SubgraphQueryRequest {
    fn from(document: String) -> Self {
        Self::new(document)
    }
}

/// Response payload for the canonical totals query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalsResponse {
    /// Totals rows returned by the subgraph.
    pub totals: Vec<Total>,
}

/// Aggregate totals row returned by the canonical totals query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Total {
    /// Number of tokens represented in the indexed dataset.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub tokens: String,
    /// Number of orders represented in the indexed dataset.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub orders: String,
    /// Number of unique traders represented in the indexed dataset.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub traders: String,
    /// Number of settlements represented in the indexed dataset.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub settlements: String,
    /// Aggregate USD volume when available from the subgraph response.
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    pub volume_usd: Option<String>,
    /// Aggregate ETH volume when available from the subgraph response.
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    pub volume_eth: Option<String>,
    /// Aggregate USD fees when available from the subgraph response.
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    pub fees_usd: Option<String>,
    /// Aggregate ETH fees when available from the subgraph response.
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    pub fees_eth: Option<String>,
}

/// Response payload for the canonical daily-volume query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastDaysVolumeResponse {
    /// Daily totals ordered by descending timestamp.
    pub daily_totals: Vec<DailyTotal>,
}

/// Single daily volume row from the canonical daily-volume query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyTotal {
    /// Unix timestamp for the indexed day bucket.
    #[serde(deserialize_with = "deserialize_u64_from_string_or_number")]
    pub timestamp: u64,
    /// USD volume for the indexed day bucket when available.
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    pub volume_usd: Option<String>,
}

/// Response payload for the canonical hourly-volume query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastHoursVolumeResponse {
    /// Hourly totals ordered by descending timestamp.
    pub hourly_totals: Vec<HourlyTotal>,
}

/// Single hourly volume row from the canonical hourly-volume query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyTotal {
    /// Unix timestamp for the indexed hour bucket.
    #[serde(deserialize_with = "deserialize_u64_from_string_or_number")]
    pub timestamp: u64,
    /// USD volume for the indexed hour bucket when available.
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    pub volume_usd: Option<String>,
}

fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    value_to_string(value).map_err(serde::de::Error::custom)
}

fn deserialize_optional_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    value
        .map(value_to_string)
        .transpose()
        .map_err(serde::de::Error::custom)
}

fn deserialize_u64_from_string_or_number<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let normalized = value_to_string(value).map_err(serde::de::Error::custom)?;
    normalized.parse::<u64>().map_err(serde::de::Error::custom)
}

fn value_to_string(value: serde_json::Value) -> Result<String, &'static str> {
    match value {
        serde_json::Value::String(value) => Ok(value),
        serde_json::Value::Number(value) => Ok(value.to_string()),
        _ => Err("expected string or number"),
    }
}
