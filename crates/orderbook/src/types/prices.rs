use serde::{Deserialize, Serialize};

use super::Amount;

/// Native-price response from `/api/v1/token/{token}/native_price`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NativePriceResponse {
    /// Token price quoted in the chain's native asset.
    pub price: f64,
}

impl NativePriceResponse {
    /// Creates a native-price response for the supplied numeric quote.
    #[must_use]
    pub const fn new(price: f64) -> Self {
        Self { price }
    }
}

/// Total-surplus response from `/api/v1/users/{owner}/total_surplus`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TotalSurplus {
    /// Total surplus value in the upstream decimal-string wire shape,
    /// denominated in the chain's native-token base units (wei, 18 decimals).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_surplus: Option<Amount>,
}

impl TotalSurplus {
    /// Creates a total-surplus response from a typed amount.
    #[must_use]
    pub const fn new(total_surplus: Amount) -> Self {
        Self {
            total_surplus: Some(total_surplus),
        }
    }
}

/// Full app-data response from the orderbook app-data endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AppDataObject {
    /// Full serialized app-data payload.
    pub full_app_data: String,
}

impl AppDataObject {
    /// Creates an app-data response from an already-serialized payload.
    #[must_use]
    pub fn new(full_app_data: impl Into<String>) -> Self {
        Self {
            full_app_data: full_app_data.into(),
        }
    }
}
