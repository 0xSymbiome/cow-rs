#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Shared `CoW` Protocol core types, validation helpers, configuration, and
//! runtime-neutral traits used across the `cow-sdk` crate family.

#![warn(missing_docs)]

/// Canonical cancellation combinator for long-running SDK futures.
pub mod cancellation;
/// Environment, address-book, and HTTP client policy types shared across crates.
pub mod config;
/// Common validation and configuration errors used by the foundational crates.
pub mod errors;
/// Typed redaction wrapper for secret-bearing configuration fields.
pub mod redaction;
/// Runtime-neutral signer, provider, and typed-data trait contracts.
pub mod traits;
/// Async HTTP transport injection point and native [`reqwest`] default.
pub mod transport;
/// Strongly typed user-domain values used across the SDK surface.
pub mod types;
/// Shared validation-failure and transport-classification enums.
pub mod validation;

pub use cancellation::{Cancellable, Cancelled, WithCancellation};
pub use config::{
    AddressPerChain, ApiBaseUrls, ApiContext, CowEnv, DEFAULT_HTTP_TIMEOUT,
    DEFAULT_MAX_RESPONSE_BYTES, ExternalHostPolicy, HostPolicyError, HttpClientPolicy,
    MAX_VALID_TO_EPOCH, NATIVE_CURRENCY_ADDRESS, ProtocolOptions, SupportedChainId,
    UrlParseFailureClass, canonical_orderbook_hosts, canonical_subgraph_hosts,
    default_api_base_urls, validate_external_service_url, wrapped_native_token,
};
pub use errors::{CoreError, ErrorClass, ValidationError};
pub use redaction::{
    REDACTED_PLACEHOLDER, REDACTED_RESPONSE_BODY_MAX_BYTES, RESPONSE_BODY_TRUNCATION_MARKER,
    Redacted, RedactedOptionalUrlMap, RedactedUrlMap, redact_response_body,
};

/// Cooperative cancellation token propagated through SDK long-running operations.
///
/// Re-exported from [`tokio_util::sync::CancellationToken`] so every public
/// crate in the workspace routes cancellation through a single typed surface
/// and avoids pulling a direct `tokio-util` dependency on the downstream
/// consumer.
pub use tokio_util::sync::CancellationToken;
pub use traits::{
    BlockInfo, ContractCall, DigestSigner, LogProvider, Provider, Signer, SigningProvider,
    TransactionBroadcast, TransactionReceipt, TransactionRequest, TransactionStatus,
    TypedDataDomain, TypedDataEnvelope, TypedDataField, TypedDataPayload, TypedDataSigner,
    TypedDataTypes, UserRejection,
};
pub use transport::{HttpTransport, TransportError, TransportResponse};

/// The [`async_trait`](macro@async_trait) attribute macro, re-exported for
/// implementors of the object-safe [`HttpTransport`] seam.
///
/// `HttpTransport` is dispatched behind `Arc<dyn HttpTransport>`, which native
/// `async fn` in traits cannot express, so implementors annotate their `impl`
/// with this macro. Re-exporting it here means a downstream implementor does
/// not declare a separate `async-trait` dependency at a matching version,
/// mirroring how `serde` re-exports its derive.
pub use async_trait::async_trait;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub use transport::{FetchTransport, FetchTransportConfig};
#[cfg(not(target_arch = "wasm32"))]
pub use transport::{ReqwestTransport, ReqwestTransportConfig};
pub use types::{
    Address, Amount, Amounts, AppCode, AppCodeError, AppDataHash, BlockHash, BuyTokenDestination,
    ChainId, Costs, FeeComponent, Hash32, HexData, LogBlockSelector, LogMeta, LogQuery, NetworkFee,
    OrderData, OrderDigest, OrderKind, OrderUid, QuoteAmountsAndCosts, RawLog, SellTokenSource,
    TokenInfo, TransactionHash, VALID_TO_MAX_RELATIVE_SECONDS, VALID_TO_MIN_RELATIVE_SECONDS,
    ValidTo,
};
pub use validation::{TransportErrorClass, ValidationReason};

// Macro-support items only: gives the `address!` literal macro a stable
// `$crate::__private::alloy_primitives` expansion path inside downstream
// crates that do not depend on `alloy-primitives` directly, plus the
// compile-time literal guard the macro expands. Nested in a module (rather
// than re-exported at the crate root) so it never becomes the shortest
// visible path rustc picks when rendering diagnostics. Not public API.
#[doc(hidden)]
pub mod __private {
    pub use alloy_primitives;

    /// Returns whether an address literal is already in the protocol-canonical
    /// lowercase wire form.
    ///
    /// The `address!` macro requires the lowercase form because an EIP-55
    /// checksum cannot be verified during const evaluation, so a mixed-case
    /// literal is rejected outright rather than accepted unchecked. The `0x`
    /// prefix is ignored; every other byte must not be ASCII uppercase.
    ///
    /// This is the pure predicate behind [`assert_lowercase_address_literal`].
    /// It is unit-tested at runtime so the contract is covered without
    /// snapshotting a toolchain-version-specific const-evaluation panic.
    #[must_use]
    pub const fn is_lowercase_address_literal(literal: &str) -> bool {
        let bytes = literal.as_bytes();
        let mut index = if bytes.len() >= 2 && bytes[0] == b'0' && bytes[1] == b'x' {
            2
        } else {
            0
        };
        while index < bytes.len() {
            if bytes[index].is_ascii_uppercase() {
                return false;
            }
            index += 1;
        }
        true
    }

    /// Compile-time guard expanded by the [`address!`](crate::address) macro.
    ///
    /// # Panics
    ///
    /// Panics during const evaluation — surfacing as a build error, never at
    /// runtime — when the literal is not in the lowercase wire form
    /// ([`is_lowercase_address_literal`] returns `false`).
    pub const fn assert_lowercase_address_literal(literal: &str) {
        assert!(
            is_lowercase_address_literal(literal),
            "address! takes the lowercase wire form: an EIP-55 checksum cannot be verified at compile time, so mixed-case literals fail closed; lowercase the literal"
        );
    }
}

#[cfg(test)]
mod address_literal_guard_tests {
    use super::__private::is_lowercase_address_literal;

    #[test]
    fn lowercase_wire_form_is_accepted() {
        // The CoW vault relayer in its canonical lowercase wire form.
        assert!(is_lowercase_address_literal(
            "0xc92e8bdf79f0507f65a392b0ab4667716bfe0110"
        ));
        // Prefix-only and empty literals carry no uppercase, so they pass the
        // case guard; length and hex validity are enforced downstream by alloy.
        assert!(is_lowercase_address_literal("0x"));
        assert!(is_lowercase_address_literal(""));
    }

    #[test]
    fn mixed_case_literal_is_rejected() {
        // The same relayer address with the leading nibble case-flipped, and a
        // fully checksummed spelling: both must be rejected, because the macro
        // cannot verify an EIP-55 checksum during const evaluation.
        assert!(!is_lowercase_address_literal(
            "0xc92E8bdf79f0507f65a392b0ab4667716BFE0110"
        ));
        assert!(!is_lowercase_address_literal(
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
        ));
    }
}
