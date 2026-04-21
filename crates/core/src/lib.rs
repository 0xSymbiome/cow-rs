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
    AddressPerChain, ApiBaseUrls, ApiContext, CowEnv, DEFAULT_HTTP_TIMEOUT, ENVS_LIST,
    EVM_NATIVE_CURRENCY_ADDRESS, HttpClientPolicy, MAX_VALID_TO_EPOCH, ProtocolOptions,
    SupportedChainId, default_api_base_urls, wrapped_native_token,
};
pub use errors::{CoreError, ValidationError};
pub use redaction::{REDACTED_PLACEHOLDER, Redacted};

/// Cooperative cancellation token propagated through SDK long-running operations.
///
/// Re-exported from [`tokio_util::sync::CancellationToken`] so every public
/// crate in the workspace routes cancellation through a single typed surface
/// and avoids pulling a direct `tokio-util` dependency on the downstream
/// consumer.
pub use tokio_util::sync::CancellationToken;
pub use traits::{
    AsyncProvider, AsyncSigner, BlockInfo, ContractCall, ContractHandle, GraphTransport,
    PinningTransport, Provider, Signer, TransactionReceipt, TransactionRequest, TypedDataDomain,
    TypedDataEnvelope, TypedDataField, TypedDataPayload, TypedDataTypes,
};
pub use transport::{HttpTransport, TransportError};
#[cfg(not(target_arch = "wasm32"))]
pub use transport::{ReqwestTransport, ReqwestTransportConfig};
pub use types::{
    Address, Amount, Amounts, AppDataHash, AppDataHex, BlockHash, BuyTokenDestination, ChainId,
    Costs, DecimalAmount, FeeComponent, Hash32, HexData, NetworkFee, ORDER_TYPE_FIELD_NAMES, Order,
    OrderDigest, OrderKind, OrderModel, OrderUid, QUOTE_AMOUNT_STAGE_NAMES, QuoteAmountsAndCosts,
    QuoteModel, QuoteRequest, QuoteResponse, SellTokenSource, SignedAmount, TokenInfo, Trade,
    TradeModel, TransactionHash, UnsignedOrder, VALID_TO_MAX_RELATIVE_SECONDS,
    VALID_TO_MIN_RELATIVE_SECONDS, ValidTo, addresses_equal, token_id,
};
pub use validation::{TransportErrorClass, ValidationReason};
