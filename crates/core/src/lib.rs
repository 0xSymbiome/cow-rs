#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]

//! Shared `CoW` Protocol core types, validation helpers, configuration, and
//! runtime-neutral traits used across the `cow-sdk` crate family.

#![warn(missing_docs)]

/// Canonical cancellation combinator for long-running SDK futures.
pub mod cancellation;
/// Environment, address-book, and HTTP client policy types shared across crates.
pub mod config;
/// Common validation and configuration errors used by the foundational crates.
pub mod errors;
/// Convenient prelude that brings the identity extension traits into scope.
///
/// Callsites bringing the prelude into scope keep their
/// `Address::new(value)` / `Hash32Ext::as_str(&value)` style accessors
/// stable across the upcoming alloy-primitive collapse for the cow
/// identity newtypes (Stage A of ADR 0052).
pub mod prelude;
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
    AddressPerChain, ApiBaseUrls, ApiContext, CowEnv, DEFAULT_HTTP_TIMEOUT, ENVS_LIST,
    EVM_NATIVE_CURRENCY_ADDRESS, ExternalHostPolicy, HostPolicyError, HttpClientPolicy,
    MAX_VALID_TO_EPOCH, ProtocolOptions, SupportedChainId, UrlParseFailureClass,
    canonical_orderbook_hosts, canonical_subgraph_hosts, default_api_base_urls,
    validate_external_service_url, wrapped_native_token,
};
pub use errors::{CoreError, ValidationError};
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
    AsyncDigestSigner, AsyncEip1193, AsyncOwner, AsyncProvider, AsyncSigner, AsyncSigningProvider,
    AsyncTypedDataSigner, BlockInfo, ContractCall, ContractHandle, GraphTransport,
    PinningTransport, Provider, Signer, TransactionBroadcast, TransactionReceipt,
    TransactionRequest, TransactionStatus, TypedDataDomain, TypedDataEnvelope, TypedDataField,
    TypedDataPayload, TypedDataTypes,
};
pub use transport::{HttpTransport, TransportError};
#[cfg(not(target_arch = "wasm32"))]
pub use transport::{ReqwestTransport, ReqwestTransportConfig};
pub use types::{
    Address, Amount, Amounts, AppDataHash, AppDataHex, BlockHash, BuyTokenDestination, ChainId,
    Costs, DecimalAmount, FeeComponent, Hash32, HexData, NetworkFee, ORDER_TYPE_FIELD_NAMES, Order,
    OrderDigest, OrderKind, OrderUid, QUOTE_AMOUNT_STAGE_NAMES, QuoteAmountsAndCosts, QuoteRequest,
    QuoteResponse, SellTokenSource, SignedAmount, TokenInfo, Trade, TradeModel, TransactionHash,
    UnsignedOrder, VALID_TO_MAX_RELATIVE_SECONDS, VALID_TO_MIN_RELATIVE_SECONDS, ValidTo,
    addresses_equal, token_id,
};
pub use validation::{TransportErrorClass, ValidationReason};
