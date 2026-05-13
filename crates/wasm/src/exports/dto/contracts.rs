use cow_sdk_pure_helpers as pure;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[cfg(feature = "trading")]
use super::OrderInput;

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

/// Contract-read callback request.
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ContractCallDto {
    /// Target contract address.
    pub address: String,
    /// ABI method name.
    pub method: String,
    /// JSON ABI fragment.
    pub abi_json: String,
    /// JSON-encoded function arguments.
    pub args_json: String,
}

#[cfg(feature = "trading")]
impl From<&cow_sdk_core::ContractCall> for ContractCallDto {
    fn from(value: &cow_sdk_core::ContractCall) -> Self {
        Self {
            address: value.address.as_str().to_owned(),
            method: value.method.clone(),
            abi_json: value.abi_json.clone(),
            args_json: value.args_json.clone(),
        }
    }
}

/// Transaction request DTO returned by transaction builders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequestDto {
    /// Destination address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Hex-encoded calldata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Native value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Gas limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_limit: Option<String>,
}

impl From<&cow_sdk_core::TransactionRequest> for TransactionRequestDto {
    fn from(value: &cow_sdk_core::TransactionRequest) -> Self {
        Self {
            to: value.to.as_ref().map(|address| address.as_str().to_owned()),
            data: value.data.as_ref().map(ToString::to_string),
            value: value.value.as_ref().map(ToString::to_string),
            gas_limit: value.gas_limit.as_ref().map(ToString::to_string),
        }
    }
}

/// Native-currency sell transaction bundle.
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct BuiltSellNativeCurrencyTxDto {
    /// Deterministic order UID.
    #[serde(rename = "orderUid")]
    pub order_uid: String,
    /// Transaction request to submit.
    pub transaction: TransactionRequestDto,
    /// Unsigned order encoded by the transaction.
    pub order_to_sign: OrderInput,
    /// Effective order owner.
    pub from: String,
}
