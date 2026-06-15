#![cfg_attr(doctest, doc = include_str!("../README.md"))]

//! Alloy-backed read-only `Provider` adapter for the `CoW` Protocol Rust SDK.
//!
//! [`RpcAlloyProvider`] wraps an Alloy `DynProvider` and exposes it through
//! [`cow_sdk_core::Provider`]. The adapter is read-only: it does not
//! implement signer creation, synchronous provider traits, or signer traits.
//!
//! ```rust,no_run
//! use cow_sdk_alloy_provider::RpcAlloyProvider;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let provider = RpcAlloyProvider::builder()
//!     .http("https://example.invalid/rpc")?
//!     .build()?;
//! # let _ = provider;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![allow(
    clippy::redundant_pub_crate,
    reason = "helper modules keep crate-private visibility explicit while __seam exposes public wrappers"
)]

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
mod retry;

#[cfg(not(target_arch = "wasm32"))]
pub use builder::{
    HttpTransport, RpcAlloyProviderBuilder, RpcAlloyProviderBuilderError, TransportState,
    TransportUnset,
};
#[cfg(not(target_arch = "wasm32"))]
pub use error::{ProviderError, ProviderErrorClass};
#[cfg(not(target_arch = "wasm32"))]
pub use provider::RpcAlloyProvider;
#[cfg(not(target_arch = "wasm32"))]
pub use retry::RetryConfig;

/// Hidden inter-crate seam for sibling `cow-rs` Alloy adapter crates.
///
/// This is not a stable consumer API. Anything exported here may change without
/// notice; it exists so sibling adapter crates can reuse the reviewed address,
/// request, block, receipt, and transport-classification helpers.
///
/// See the Stability section of `docs/adr/0035-alloy-provider-adapter.md` for
/// the semver posture.
#[cfg(not(target_arch = "wasm32"))]
#[doc(hidden)]
pub mod __seam {
    use cow_sdk_core::{BlockInfo, TransactionReceipt};

    pub use crate::error::__transport_classification::RpcErrorClassification;

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
    #[must_use]
    pub fn alloy_to_cow_receipt(
        receipt: &alloy_rpc_types_eth::TransactionReceipt,
    ) -> TransactionReceipt {
        crate::conversion::alloy_to_cow_receipt(receipt)
    }

    /// Converts an Alloy block response into the core block-info contract.
    #[must_use]
    pub fn alloy_to_cow_block_info(block: &alloy_rpc_types_eth::Block) -> BlockInfo {
        crate::conversion::alloy_to_cow_block_info(block)
    }

    /// Classifies an Alloy JSON-RPC or transport error.
    #[must_use]
    pub fn rpc_error_to_class_and_detail(
        error: alloy_transport::TransportError,
    ) -> RpcErrorClassification {
        crate::error::__transport_classification::rpc_error_to_class_and_detail(error)
    }

    /// Executes the canonical read-contract algorithm.
    ///
    /// Inter-crate seam entry; not part of the semver-stable consumer API.
    /// Sibling adapter crates use this to reuse the dynamic-ABI encode,
    /// dispatch, decode, and JSON serialization path without copying it.
    /// Errors propagate as [`crate::ProviderError`] and may be lifted
    /// into a sibling adapter's error type through that crate's
    /// [`From`] impl. The argument shape may change in any minor release.
    pub async fn execute_read_contract(
        provider: &alloy_provider::DynProvider<alloy_network::Ethereum>,
        request: &cow_sdk_core::ContractCall,
    ) -> Result<String, crate::ProviderError> {
        crate::read_contract::execute_read_contract(provider, request).await
    }

    /// Converts a core [`cow_sdk_core::LogQuery`] into an Alloy `eth_getLogs`
    /// filter.
    ///
    /// Inter-crate seam entry; not part of the semver-stable consumer API. The
    /// sibling umbrella adapter consumes this so its `LogProvider` implementation
    /// reuses the reviewed `LogQuery` → filter mapping without copying it.
    #[must_use]
    pub fn cow_log_query_to_alloy_filter(
        query: &cow_sdk_core::LogQuery,
    ) -> alloy_rpc_types_eth::Filter {
        crate::conversion::cow_log_query_to_alloy_filter(query)
    }

    /// Converts an Alloy log into the core [`cow_sdk_core::RawLog`] contract.
    ///
    /// Inter-crate seam entry; not part of the semver-stable consumer API. The
    /// sibling umbrella adapter consumes this so its `LogProvider` implementation
    /// reuses the reviewed Alloy-log → `RawLog` mapping without copying it.
    #[must_use]
    pub fn alloy_log_to_cow_raw_log(log: &alloy_rpc_types_eth::Log) -> cow_sdk_core::RawLog {
        crate::conversion::alloy_log_to_cow_raw_log(log)
    }

    /// Builds the opt-in RPC retry/backoff transport layer for a
    /// [`crate::RetryConfig`].
    ///
    /// Inter-crate seam entry; not part of the semver-stable consumer API. The
    /// sibling umbrella adapter consumes this so its layered JSON-RPC client uses
    /// the same retry policy and internal compute-units budget as the read-only
    /// provider leaf, without redefining either.
    #[must_use]
    pub fn retry_backoff_layer(
        config: &crate::RetryConfig,
    ) -> alloy_transport::layers::RetryBackoffLayer {
        config.backoff_layer()
    }
}
