//! Browser-wallet error types and RPC error normalization.

use cow_sdk_core::{ChainId, CoreError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// Serialized JSON-RPC error payload returned by an EIP-1193 wallet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcErrorPayload {
    /// Numeric wallet or RPC error code.
    pub code: i32,
    /// Human-readable wallet or RPC error message.
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional provider-specific error data.
    pub data: Option<Value>,
}

/// Errors produced by typed browser-wallet discovery, session, provider, and signer flows.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum BrowserWalletError {
    /// No injected browser wallet provider is available in the current runtime.
    #[error("wallet provider is unavailable")]
    WalletUnavailable,
    /// Discovery found more than one candidate and requires an explicit wallet selection.
    #[error(
        "wallet discovery requires explicit provider selection because {candidates} injected wallets were found"
    )]
    DiscoverySelectionRequired {
        /// Number of candidates returned by discovery.
        candidates: usize,
    },
    /// A requested discovery index is outside the available wallet range.
    #[error(
        "wallet discovery selection index {index} is out of range for {candidates} injected wallets"
    )]
    DiscoverySelectionOutOfRange {
        /// Requested wallet index.
        index: usize,
        /// Number of available discovery candidates.
        candidates: usize,
    },
    /// The wallet explicitly rejected a user-authorized request.
    #[error("wallet request `{method}` was rejected by the user ({code}): {message}")]
    UserRejectedRequest {
        /// RPC method that was rejected.
        method: String,
        /// Provider error code.
        code: i32,
        /// Provider error message.
        message: String,
    },
    /// The wallet reported that it is disconnected from all chains.
    #[error(
        "wallet request `{method}` failed because the provider is disconnected ({code}): {message}"
    )]
    Disconnected {
        /// RPC method that failed.
        method: String,
        /// Provider error code.
        code: i32,
        /// Provider error message.
        message: String,
    },
    /// The wallet reported that the currently connected chain is incompatible with the request.
    #[error(
        "wallet request `{method}` failed because the current chain is not connected ({code}): {message}"
    )]
    WrongChain {
        /// RPC method that failed.
        method: String,
        /// Provider error code.
        code: i32,
        /// Provider error message.
        message: String,
    },
    /// The requested chain has not been added to the wallet yet.
    #[error(
        "wallet request `{method}` failed because chain {chain_id} is not added ({code}): {message}"
    )]
    ChainNotAdded {
        /// Chain id requested by the wallet call.
        chain_id: ChainId,
        /// RPC method that failed.
        method: String,
        /// Provider error code.
        code: i32,
        /// Provider error message.
        message: String,
    },
    /// The typed add-chain input is invalid before any wallet request is attempted.
    #[error("wallet chain configuration for chain {chain_id} is invalid: {message}")]
    InvalidChainConfiguration {
        /// Chain id referenced by the configuration.
        chain_id: ChainId,
        /// Validation failure description.
        message: String,
    },
    /// The wallet does not support the requested RPC method.
    #[error("wallet method `{method}` is unsupported: {message}")]
    UnsupportedRpcMethod {
        /// Unsupported RPC method.
        method: String,
        /// Provider-supplied failure description.
        message: String,
    },
    /// The wallet returned a response that does not match the typed contract.
    #[error("wallet response for `{method}` is malformed: {message}")]
    MalformedResponse {
        /// RPC method whose response could not be decoded.
        method: String,
        /// Decode or validation failure description.
        message: String,
    },
    /// An unclassified wallet or RPC error payload.
    #[error("wallet rpc error for `{method}` ({code}): {message}")]
    Rpc {
        /// RPC method that failed.
        method: String,
        /// Provider error code.
        code: i32,
        /// Provider error message.
        message: String,
        /// Optional provider-specific error data.
        data: Option<Value>,
    },
    /// JavaScript interop or DOM interaction failed in the browser runtime.
    #[error("wallet JS interop error: {message}")]
    JsInterop {
        /// Interop failure description.
        message: String,
    },
    /// JSON serialization, ABI conversion, or typed-data encoding failed locally.
    #[error("wallet serialization error: {message}")]
    Serialization {
        /// Serialization or local encoding failure description.
        message: String,
    },
    /// Shared core type or validation error.
    #[error(transparent)]
    Core(#[from] CoreError),
}

impl BrowserWalletError {
    pub(crate) fn from_rpc(
        method: &str,
        payload: RpcErrorPayload,
        requested_chain: Option<ChainId>,
    ) -> Self {
        match payload.code {
            4001 => Self::UserRejectedRequest {
                method: method.to_owned(),
                code: payload.code,
                message: payload.message,
            },
            4900 => Self::Disconnected {
                method: method.to_owned(),
                code: payload.code,
                message: payload.message,
            },
            4901 => Self::WrongChain {
                method: method.to_owned(),
                code: payload.code,
                message: payload.message,
            },
            4902 => Self::ChainNotAdded {
                chain_id: requested_chain.unwrap_or_default(),
                method: method.to_owned(),
                code: payload.code,
                message: payload.message,
            },
            -32601 => Self::UnsupportedRpcMethod {
                method: method.to_owned(),
                message: payload.message,
            },
            _ => Self::Rpc {
                method: method.to_owned(),
                code: payload.code,
                message: payload.message,
                data: payload.data,
            },
        }
    }

    pub(crate) fn malformed_response(method: &str, message: impl Into<String>) -> Self {
        Self::MalformedResponse {
            method: method.to_owned(),
            message: message.into(),
        }
    }

    pub(crate) fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }

    pub(crate) fn discovery_selection_required(candidates: usize) -> Self {
        Self::DiscoverySelectionRequired { candidates }
    }

    pub(crate) fn discovery_selection_out_of_range(index: usize, candidates: usize) -> Self {
        Self::DiscoverySelectionOutOfRange { index, candidates }
    }

    pub(crate) fn invalid_chain_configuration(
        chain_id: ChainId,
        message: impl Into<String>,
    ) -> Self {
        Self::InvalidChainConfiguration {
            chain_id,
            message: message.into(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn js(message: impl Into<String>) -> Self {
        Self::JsInterop {
            message: message.into(),
        }
    }
}
