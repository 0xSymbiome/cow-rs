use serde::{Deserialize, Serialize};

use crate::types::Address;
/// Typed contract-read request used by runtime-neutral providers.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractCall {
    /// Target contract address.
    pub address: Address,
    /// ABI method name to invoke.
    pub method: String,
    /// JSON ABI fragment describing the contract or function.
    pub abi_json: String,
    /// JSON-encoded function arguments.
    pub args_json: String,
}

impl ContractCall {
    /// Creates a typed contract-read request.
    #[inline]
    #[must_use]
    pub const fn new(
        address: Address,
        method: String,
        abi_json: String,
        args_json: String,
    ) -> Self {
        Self {
            address,
            method,
            abi_json,
            args_json,
        }
    }
}

/// Contract handle returned by providers that support typed contract creation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractHandle {
    /// Target contract address.
    pub address: Address,
    /// JSON ABI for the contract handle.
    pub abi_json: String,
}

impl ContractHandle {
    /// Creates a typed contract handle.
    #[inline]
    #[must_use]
    pub const fn new(address: Address, abi_json: String) -> Self {
        Self { address, abi_json }
    }
}
