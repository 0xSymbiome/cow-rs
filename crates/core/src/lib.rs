//! Shared `CoW` Protocol core types, validation helpers, configuration, and
//! runtime-neutral traits used across the `cow-sdk` crate family.

#![warn(missing_docs)]

/// Environment, address-book, and HTTP client policy types shared across crates.
pub mod config;
/// Common validation and configuration errors used by the foundational crates.
pub mod errors;
/// Typed redaction wrapper for secret-bearing configuration fields.
pub mod redaction;
/// Runtime-neutral signer, provider, and typed-data trait contracts.
pub mod traits;
/// Strongly typed user-domain values used across the SDK surface.
pub mod types;

pub use config::{
    AddressPerChain, ApiBaseUrls, ApiContext, CowEnv, DEFAULT_HTTP_TIMEOUT, ENVS_LIST,
    EVM_NATIVE_CURRENCY_ADDRESS, HttpClientPolicy, MAX_VALID_TO_EPOCH, ProtocolOptions,
    SupportedChainId, default_api_base_urls, eth_flow_contract_address,
    settlement_contract_address, vault_relayer_address, wrapped_native_token,
};
pub use errors::{CoreError, ValidationError};
pub use redaction::{REDACTED_PLACEHOLDER, Redacted};
pub use traits::{
    AsyncProvider, AsyncSigner, BlockInfo, ContractCall, ContractHandle, GraphTransport,
    HttpTransport, PinningTransport, Provider, Signer, TransactionReceipt, TransactionRequest,
    TypedDataDomain, TypedDataEnvelope, TypedDataField, TypedDataPayload, TypedDataTypes,
};
pub use types::{
    Address, Amount, Amounts, AppDataHash, AppDataHex, AtomAmount, BlockHash, ChainId, Costs,
    DecimalAmount, FeeComponent, Hash32, HexData, NetworkFee, ORDER_TYPE_FIELD_NAMES, Order,
    OrderBalance, OrderDigest, OrderKind, OrderModel, OrderUid, QUOTE_AMOUNT_STAGE_NAMES,
    QuoteAmountsAndCosts, QuoteModel, QuoteRequest, QuoteResponse, SignedAmount, TokenInfo, Trade,
    TradeModel, TransactionHash, UnsignedOrder, VALID_TO_MAX_RELATIVE_SECONDS,
    VALID_TO_MIN_RELATIVE_SECONDS, ValidTo, addresses_equal, token_id,
};
