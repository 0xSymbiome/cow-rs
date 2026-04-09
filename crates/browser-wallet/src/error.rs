use cow_sdk_core::{ChainId, CoreError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcErrorPayload {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum BrowserWalletError {
    #[error("wallet provider is unavailable")]
    WalletUnavailable,
    #[error("wallet request `{method}` was rejected by the user ({code}): {message}")]
    UserRejectedRequest {
        method: String,
        code: i32,
        message: String,
    },
    #[error(
        "wallet request `{method}` failed because the provider is disconnected ({code}): {message}"
    )]
    Disconnected {
        method: String,
        code: i32,
        message: String,
    },
    #[error(
        "wallet request `{method}` failed because the current chain is not connected ({code}): {message}"
    )]
    WrongChain {
        method: String,
        code: i32,
        message: String,
    },
    #[error(
        "wallet request `{method}` failed because chain {chain_id} is not added ({code}): {message}"
    )]
    ChainNotAdded {
        chain_id: ChainId,
        method: String,
        code: i32,
        message: String,
    },
    #[error("wallet method `{method}` is unsupported: {message}")]
    UnsupportedRpcMethod { method: String, message: String },
    #[error("wallet response for `{method}` is malformed: {message}")]
    MalformedResponse { method: String, message: String },
    #[error("wallet rpc error for `{method}` ({code}): {message}")]
    Rpc {
        method: String,
        code: i32,
        message: String,
        data: Option<Value>,
    },
    #[error("wallet JS interop error: {message}")]
    JsInterop { message: String },
    #[error("wallet serialization error: {message}")]
    Serialization { message: String },
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

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn js(message: impl Into<String>) -> Self {
        Self::JsInterop {
            message: message.into(),
        }
    }
}
