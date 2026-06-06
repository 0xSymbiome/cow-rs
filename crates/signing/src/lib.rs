#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]

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
/// EIP-1271 custom-signature provider boundary.
pub mod eip1271;
/// Signing crate error types.
pub mod errors;
/// Order signing and order-id generation helpers.
pub mod order_signing;

#[cfg(feature = "in-memory-cache")]
pub use cache::{
    Clock, DEFAULT_EIP1271_VERIFICATION_CACHE_CAPACITY, DEFAULT_EIP1271_VERIFICATION_CACHE_TTL,
    InMemoryEip1271VerificationCache, SystemClock,
};
pub use cache::{Eip1271VerificationCache, NoopEip1271VerificationCache};
pub use cancellation::{
    ORDER_CANCELLATIONS_PRIMARY_TYPE, order_cancellation_typed_data_payload,
    order_cancellations_typed_data_payload, sign_order_cancellation,
    sign_order_cancellation_with_scheme, sign_order_cancellations,
    sign_order_cancellations_with_scheme,
};
pub use cow_sdk_contracts::SigningScheme;
pub use cow_sdk_contracts::{
    Eip1271VerificationRequest, verify_eip1271_signature, verify_eip1271_signature_cached,
};
pub use domain::{
    ORDER_PRIMARY_TYPE, OrderTypedData, cancellation_fields, domain, domain_fields,
    domain_separator, domain_separator_for, order_fields, order_typed_data,
    order_typed_data_payload,
};
pub use eip1271::{
    Eip1271SignatureError, Eip1271SignatureProvider, OnchainOrder, OrderAndSignature,
};
pub use errors::SigningError;
pub use order_signing::{
    GeneratedOrderId, SigningResult, eip1271_signature_payload, generate_order_id, sign_order,
    sign_order_with_scheme,
};
