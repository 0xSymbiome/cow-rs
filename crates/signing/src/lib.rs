//! Explicit `CoW` Protocol signing helpers.
//!
//! The durable typed-data boundary is `cow_sdk_core::TypedDataPayload`.
//! This crate owns explicit payload construction for `CoW` order and order
//! cancellation signing so runtime adapters such as `cow-sdk-browser-wallet`
//! can transport typed-data payloads without reconstructing structure from
//! field-name heuristics.

#![warn(missing_docs)]

/// EIP-1271 signature-verification cache trait and default impls.
pub mod cache;
/// Order-cancellation signing helpers.
pub mod cancellation;
/// Typed-data domain and payload construction helpers.
pub mod domain;
/// Signing crate error types.
pub mod errors;
/// Order signing and order-id generation helpers.
pub mod order_signing;

pub use cache::{
    DEFAULT_EIP1271_VERIFICATION_CACHE_CAPACITY, DEFAULT_EIP1271_VERIFICATION_CACHE_TTL,
    Eip1271VerificationCache, InMemoryEip1271VerificationCache, NoopEip1271VerificationCache,
};
pub use cancellation::{
    ORDER_CANCELLATIONS_PRIMARY_TYPE, order_cancellation_typed_data_payload,
    order_cancellations_typed_data_payload, sign_order_cancellation, sign_order_cancellation_async,
    sign_order_cancellation_with_scheme, sign_order_cancellation_with_scheme_async,
    sign_order_cancellations, sign_order_cancellations_async, sign_order_cancellations_with_scheme,
    sign_order_cancellations_with_scheme_async,
};
pub use cow_sdk_contracts::SigningScheme;
pub use cow_sdk_contracts::{
    Eip1271VerificationRequest, verify_eip1271_signature, verify_eip1271_signature_async,
};
pub use domain::{
    ORDER_PRIMARY_TYPE, OrderTypedData, cancellation_fields, domain_fields, domain_separator,
    domain_separator_for, get_domain, order_fields, order_typed_data, order_typed_data_payload,
};
pub use errors::SigningError;
pub use order_signing::{
    GeneratedOrderId, SigningResult, eip1271_signature_payload, generate_order_id, sign_order,
    sign_order_async, sign_order_with_scheme, sign_order_with_scheme_async,
};
