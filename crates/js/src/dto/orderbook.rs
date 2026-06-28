#[cfg(all(target_arch = "wasm32", feature = "orderbook"))]
use crate::helpers::errors::PureError;
#[cfg(feature = "orderbook")]
use serde::{Deserialize, Serialize};
#[cfg(all(target_arch = "wasm32", feature = "orderbook"))]
use serde_json::Value;

#[cfg(feature = "orderbook")]
use cow_sdk_core::{BuyTokenDestination, OrderKind, SellTokenSource};

#[cfg(all(target_arch = "wasm32", feature = "orderbook"))]
use crate::exports::errors::WasmError;

// The orderbook response types (`Order`, `Trade`, `QuoteData`,
// `OrderQuoteResponse`, `CompetitionOrderStatus`, `TotalSurplus`,
// `SolverCompetitionResponse`, `AppDataObject`, and their nested types) live in
// `cow_sdk_orderbook`: each native type carries its own `tsify` boundary derive
// (gated to the wasm-bindgen npm target behind `ts-bindings`), so the `.d.ts`
// declarations are generated directly from the native types and the orderbook
// exports return them unchanged. Only the request-input DTOs and the
// signing-scheme parsers live here, because they have no native counterpart with
// the wasm-input ABI.

/// Orderbook quote request input.
#[cfg(feature = "orderbook")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "orderbook"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "orderbook"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "this wasm-boundary request input derives only PartialEq for test equality and omits Eq to match the boundary DTO derive set"
)]
pub struct OrderQuoteRequest {
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
    pub kind: OrderKind,
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
    /// Sell-token balance source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<SellTokenSource>,
    /// Buy-token balance destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<BuyTokenDestination>,
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

#[cfg(all(target_arch = "wasm32", feature = "orderbook"))]
impl OrderQuoteRequest {
    pub(crate) fn into_value(self) -> Result<Value, WasmError> {
        serde_json::to_value(self).map_err(WasmError::from)
    }
}

/// Orderbook order-creation input.
#[cfg(feature = "orderbook")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "orderbook"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "orderbook"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "this wasm-boundary order-creation input derives only PartialEq for test equality and omits Eq to match the boundary DTO derive set"
)]
pub struct OrderCreation {
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
    pub kind: OrderKind,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<SellTokenSource>,
    /// Buy-token balance destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<BuyTokenDestination>,
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

#[cfg(all(target_arch = "wasm32", feature = "orderbook"))]
impl OrderCreation {
    pub(crate) fn into_value(self) -> Result<Value, WasmError> {
        serde_json::to_value(self).map_err(WasmError::from)
    }
}

/// Pagination options shared by orderbook list helpers.
#[cfg(feature = "orderbook")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "orderbook"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "orderbook"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct PaginationOptions {
    /// Pagination offset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    /// Pagination limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Trades query accepted by `OrderBookClient.getTrades`.
#[cfg(feature = "orderbook")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "orderbook"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "orderbook"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct GetTradesRequest {
    /// Owner filter. Set exactly one of `owner` or `orderUid`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Order UID filter. Set exactly one of `owner` or `orderUid`.
    #[serde(rename = "orderUid", default, skip_serializing_if = "Option::is_none")]
    pub order_uid: Option<String>,
    /// Pagination offset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    /// Pagination limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[cfg(all(target_arch = "wasm32", feature = "orderbook"))]
pub fn orderbook_signing_scheme(
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

#[cfg(all(target_arch = "wasm32", feature = "orderbook"))]
pub fn ecdsa_signing_scheme(
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

// Guards the input-DTO -> native round-trip that the orderbook exports perform
// at runtime (`into_value()` then `from_value::<Native>` via `from_json_value`).
// A future field rename/retype on either the DTO or the native type fails here
// instead of silently at runtime. The orderbook *response* types carry Tsify on
// the native type, so the native wire contract in
// `crates/orderbook/tests/wire_contract.rs` covers the response side; these input
// round-trips are the symmetric guard on the request side.
#[cfg(all(test, target_arch = "wasm32", feature = "orderbook"))]
mod input_dto_roundtrip_tests {
    use super::{OrderCreation, OrderKind, OrderQuoteRequest};
    use wasm_bindgen_test::wasm_bindgen_test;

    // Canonical mainnet addresses: valid 20-byte payloads the native `Address`
    // deserializer accepts.
    const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    const OWNER: &str = "0x9008D19f58AAbD9eD0D60971565AA8510560ab41";

    #[wasm_bindgen_test]
    fn order_quote_request_input_round_trips_into_native_request() {
        let dto = OrderQuoteRequest {
            sell_token: WETH.to_owned(),
            buy_token: USDC.to_owned(),
            receiver: None,
            from: OWNER.to_owned(),
            kind: OrderKind::Sell,
            sell_amount_before_fee: Some("1000000000000000000".to_owned()),
            buy_amount_after_fee: None,
            valid_for: Some(1800),
            valid_to: None,
            app_data: None,
            app_data_hash: None,
            sell_token_balance: None,
            buy_token_balance: None,
            price_quality: None,
            signing_scheme: Some("eip712".to_owned()),
            onchain_order: false,
            verification_gas_limit: None,
            timeout: None,
        };

        let value = dto.into_value().expect("the quote-request DTO serializes");
        let native: cow_sdk_orderbook::OrderQuoteRequest = serde_json::from_value(value)
            .expect("the wasm quote-request DTO must round-trip into the native OrderQuoteRequest");

        // The DTO's flat `kind` + `sellAmountBeforeFee` must flatten into the
        // native tagged `OrderQuoteSide::Sell`.
        assert!(matches!(
            native.side,
            cow_sdk_orderbook::OrderQuoteSide::Sell { .. }
        ));
        assert_eq!(native.from.to_hex_string(), OWNER.to_lowercase());
    }

    #[wasm_bindgen_test]
    fn order_creation_input_round_trips_into_native_creation() {
        let dto = OrderCreation {
            sell_token: WETH.to_owned(),
            buy_token: USDC.to_owned(),
            receiver: None,
            sell_amount: "1000000000000000000".to_owned(),
            buy_amount: "2000000000".to_owned(),
            valid_to: 2_000_000_000,
            // The `(Some document, None hash)` app-data case — the simplest wire
            // shape the native untagged-enum routing accepts.
            app_data: Some("{}".to_owned()),
            app_data_hash: None,
            fee_amount: Some("0".to_owned()),
            full_balance_check: false,
            kind: OrderKind::Sell,
            partially_fillable: false,
            sell_token_balance: None,
            buy_token_balance: None,
            signing_scheme: "eip712".to_owned(),
            signature: format!("0x{}", "00".repeat(65)),
            from: OWNER.to_owned(),
            quote_id: None,
        };

        let value = dto.into_value().expect("the order-creation DTO serializes");
        let native: cow_sdk_orderbook::OrderCreation = serde_json::from_value(value)
            .expect("the wasm order-creation DTO must round-trip into the native OrderCreation");

        assert_eq!(native.valid_to, 2_000_000_000);
        assert_eq!(
            native.signing_scheme,
            cow_sdk_orderbook::SigningScheme::Eip712
        );
        assert_eq!(native.from.to_hex_string(), OWNER.to_lowercase());
    }
}
