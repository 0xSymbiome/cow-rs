use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalsResponse {
    pub totals: Vec<Total>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Total {
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub tokens: String,
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub orders: String,
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub traders: String,
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub settlements: String,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    pub volume_usd: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    pub volume_eth: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    pub fees_usd: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    pub fees_eth: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastDaysVolumeResponse {
    pub daily_totals: Vec<DailyTotal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyTotal {
    #[serde(deserialize_with = "deserialize_u64_from_string_or_number")]
    pub timestamp: u64,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    pub volume_usd: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastHoursVolumeResponse {
    pub hourly_totals: Vec<HourlyTotal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HourlyTotal {
    #[serde(deserialize_with = "deserialize_u64_from_string_or_number")]
    pub timestamp: u64,
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
