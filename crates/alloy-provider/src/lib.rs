#![cfg_attr(doctest, doc = include_str!("../README.md"))]

//! Alloy-backed read-only `AsyncProvider` adapter for the `CoW` Protocol Rust SDK.
//!
//! [`RpcAlloyProvider`] wraps an Alloy `DynProvider` and exposes it through
//! [`cow_sdk_core::AsyncProvider`]. The adapter is read-only: it does not
//! implement signer creation, synchronous provider traits, or signer traits.
//!
//! ```rust,no_run
//! use cow_sdk_alloy_provider::RpcAlloyProvider;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = RpcAlloyProvider::builder()
//!     .http("https://example.invalid/rpc")?
//!     .build()
//!     .await?;
//! # let _ = provider;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![allow(
    clippy::redundant_pub_crate,
    reason = "helper modules keep crate-private visibility explicit while __seam exposes public wrappers"
)]

#[cfg(target_arch = "wasm32")]
compile_error!(
    "the alloy / alloy-provider / alloy-signer features on cow-sdk are for native targets only; cow-sdk-alloy-provider is native-only, and wasm targets should use cow-sdk-browser-wallet for signing and consumer-supplied EIP-1193 providers for RPC reads."
);

#[cfg(not(target_arch = "wasm32"))]
mod builder;
#[cfg(not(target_arch = "wasm32"))]
mod client;
#[cfg(not(target_arch = "wasm32"))]
mod conversion;
#[cfg(not(target_arch = "wasm32"))]
mod error;
#[cfg(not(target_arch = "wasm32"))]
mod provider;
#[cfg(not(target_arch = "wasm32"))]
mod read_contract;

#[cfg(not(target_arch = "wasm32"))]
pub use builder::{
    HttpTransport, RpcAlloyProviderBuilder, RpcAlloyProviderBuilderError, TransportState,
    TransportUnset,
};
#[cfg(not(target_arch = "wasm32"))]
pub use error::{AsyncProviderError, AsyncProviderErrorClass};
#[cfg(not(target_arch = "wasm32"))]
pub use provider::RpcAlloyProvider;

/// Hidden inter-crate seam for sibling `cow-rs` Alloy adapter crates.
///
/// This is not a stable consumer API. Anything exported here may change without
/// notice; it exists so sibling adapter crates can reuse the reviewed address,
/// request, block, receipt, and transport-classification helpers.
#[cfg(not(target_arch = "wasm32"))]
#[doc(hidden)]
pub mod __seam {
    use cow_sdk_core::{BlockInfo, Redacted, TransactionReceipt, TransportErrorClass};

    use crate::AsyncProviderError;

    /// Classified Alloy JSON-RPC or transport error detail.
    #[non_exhaustive]
    pub enum RpcErrorClassification {
        /// Transport-layer classification with redacted detail.
        Transport {
            /// Shared transport class.
            class: TransportErrorClass,
            /// Redacted detail.
            detail: Redacted<String>,
        },
        /// Remote JSON-RPC payload.
        Remote {
            /// JSON-RPC error code.
            code: i64,
            /// JSON-RPC error message.
            message: String,
        },
        /// Local invariant or unsupported upstream path.
        Internal(String),
    }

    impl From<crate::error::__transport_classification::RpcErrorClassification>
        for RpcErrorClassification
    {
        fn from(
            classification: crate::error::__transport_classification::RpcErrorClassification,
        ) -> Self {
            match classification {
                crate::error::__transport_classification::RpcErrorClassification::Transport {
                    class,
                    detail,
                } => Self::Transport { class, detail },
                crate::error::__transport_classification::RpcErrorClassification::Remote {
                    code,
                    message,
                } => Self::Remote { code, message },
                crate::error::__transport_classification::RpcErrorClassification::Internal(
                    message,
                ) => Self::Internal(message),
            }
        }
    }

    /// Converts a `cow-sdk-core` address into Alloy's address type.
    pub fn cow_to_alloy_address(
        address: &cow_sdk_core::Address,
    ) -> Result<alloy_primitives::Address, AsyncProviderError> {
        crate::conversion::cow_to_alloy_address(address)
    }

    /// Converts a `cow-sdk-core` transaction hash into Alloy's hash type.
    pub fn cow_to_alloy_hash(
        transaction_hash: &cow_sdk_core::TransactionHash,
    ) -> Result<alloy_primitives::B256, AsyncProviderError> {
        crate::conversion::cow_to_alloy_hash(transaction_hash)
    }

    /// Converts a core transaction request into an Alloy transaction request.
    pub fn cow_request_to_alloy(
        request: &cow_sdk_core::TransactionRequest,
    ) -> Result<alloy_rpc_types_eth::TransactionRequest, String> {
        crate::conversion::cow_request_to_alloy(request)
    }

    /// Converts a core block tag string into Alloy's block-id type.
    pub fn cow_block_tag_to_alloy(tag: &str) -> Result<alloy_rpc_types_eth::BlockId, String> {
        crate::conversion::cow_block_tag_to_alloy(tag)
    }

    /// Converts an Alloy transaction receipt into the core receipt contract.
    pub fn alloy_to_cow_receipt(
        receipt: &alloy_rpc_types_eth::TransactionReceipt,
    ) -> Result<TransactionReceipt, AsyncProviderError> {
        crate::conversion::alloy_to_cow_receipt(receipt)
    }

    /// Converts an Alloy block response into the core block-info contract.
    pub fn alloy_to_cow_block_info(
        block: &alloy_rpc_types_eth::Block,
    ) -> Result<BlockInfo, AsyncProviderError> {
        crate::conversion::alloy_to_cow_block_info(block)
    }

    /// Classifies an Alloy JSON-RPC or transport error.
    #[must_use]
    pub fn rpc_error_to_class_and_detail(
        error: alloy_transport::TransportError,
    ) -> RpcErrorClassification {
        crate::error::__transport_classification::rpc_error_to_class_and_detail(error).into()
    }
}
