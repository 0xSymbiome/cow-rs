use crate::config::SupportedChainId;
use crate::types::{Address, Amount};

use super::transaction::{TransactionBroadcast, TransactionRequest};
use super::typed_data::TypedDataPayload;

/// Owner-address capability.
///
/// This narrow trait lets flows ask only for signer ownership when no
/// signing operation is required.
#[expect(
    async_fn_in_trait,
    reason = "the trait surface adopts native async fn in trait per ADR 0010 runtime-neutral posture; the resulting non-Send futures are covered by the workspace future_not_send allow so wasm callbacks can satisfy the same trait without an explicit Send bound"
)]
pub trait Owner {
    /// Error type returned by owner resolution.
    type Error;

    /// Returns the signer address.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when address resolution fails.
    async fn address(&self) -> Result<Address, Self::Error>;
}

/// EIP-712 typed-data signing capability.
#[expect(
    async_fn_in_trait,
    reason = "the trait surface adopts native async fn in trait per ADR 0010 runtime-neutral posture; the resulting non-Send futures are covered by the workspace future_not_send allow so wasm callbacks can satisfy the same trait without an explicit Send bound"
)]
pub trait TypedDataSigner {
    /// Error type returned by typed-data signing.
    type Error;

    /// Signs the canonical EIP-712 typed-data payload.
    ///
    /// The payload carries the domain, the full types map, the primary-type
    /// name, and the message — everything a backend needs to compute the
    /// canonical EIP-712 digest. Field-based signing is deliberately not a
    /// trait obligation: a (domain, fields, message) triple cannot name its
    /// primary type or carry nested type definitions, so it cannot express a
    /// correct digest for arbitrary payloads.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error>;
}

/// Digest-signing capability.
#[expect(
    async_fn_in_trait,
    reason = "the trait surface adopts native async fn in trait per ADR 0010 runtime-neutral posture; the resulting non-Send futures are covered by the workspace future_not_send allow so wasm callbacks can satisfy the same trait without an explicit Send bound"
)]
pub trait DigestSigner {
    /// Error type returned by digest signing.
    type Error;

    /// Signs raw digest bytes according to the backend's message-signing rules.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    async fn sign_digest(&self, digest: &[u8]) -> Result<String, Self::Error>;
}

/// Signing boundary for wallets and runtimes.
///
/// Production adapters implement this trait directly. Narrow capability
/// traits above are preferred for callback-shaped adapters that only expose
/// one signing operation.
#[expect(
    async_fn_in_trait,
    reason = "the trait surface adopts native async fn in trait per ADR 0010 runtime-neutral posture; the resulting non-Send futures are covered by the workspace future_not_send allow so wasm callbacks can satisfy the same trait without an explicit Send bound"
)]
pub trait Signer {
    /// Error type returned by signer operations.
    type Error;

    /// Returns the chain this signer signs for, when statically known.
    ///
    /// A trading flow consults this hint to fast-fail a signer/trading
    /// chain mismatch *before* signing: if the signer is statically bound
    /// to one chain but the trading client is configured for another, the
    /// produced EIP-712 signature would carry the wrong domain separator
    /// and the order would be rejected after a wasted orderbook round-trip.
    /// Catching the mismatch here keeps the failure local and typed.
    ///
    /// Returning [`None`] — the default — opts out of the check. Wallet and
    /// callback signers that learn their chain at runtime (EIP-1193
    /// providers, browser wallets, recording doubles, and the pre-sign
    /// placement stand-in) cannot name a single static chain, so they
    /// inherit the default and the flow proceeds without the hint.
    ///
    /// This method is intentionally synchronous and defaulted: it reports a
    /// construction-time fact, not a runtime query, and a signer adopts the
    /// trait without implementing it.
    #[must_use]
    fn chain_id(&self) -> Option<SupportedChainId> {
        None
    }

    /// Returns the signer address.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when address resolution fails.
    async fn address(&self) -> Result<Address, Self::Error>;
    /// Signs arbitrary bytes according to the backend's message-signing rules.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error>;
    /// Signs a transaction payload.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    async fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error>;
    /// Signs the canonical EIP-712 typed-data payload.
    ///
    /// The payload carries the domain, the full types map, the primary-type
    /// name, and the message — everything a backend needs to compute the
    /// canonical EIP-712 digest.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error>;
    /// Sends a transaction and returns the broadcast transaction hash.
    ///
    /// This confirms only that the signer backend returned a transaction hash.
    /// Use [`crate::Provider::get_transaction_receipt`] or a higher-level
    /// `cow-sdk-trading` wait helper to observe mining status and receipt
    /// fields.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when submission fails.
    async fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error>;
    /// Estimates gas for a transaction request.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when estimation fails.
    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<Amount, Self::Error>;
}

impl<T> Owner for T
where
    T: Signer,
{
    type Error = T::Error;

    async fn address(&self) -> Result<Address, Self::Error> {
        Signer::address(self).await
    }
}

impl<T> TypedDataSigner for T
where
    T: Signer,
{
    type Error = T::Error;

    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        Signer::sign_typed_data_payload(self, payload).await
    }
}

impl<T> DigestSigner for T
where
    T: Signer,
{
    type Error = T::Error;

    async fn sign_digest(&self, digest: &[u8]) -> Result<String, Self::Error> {
        Signer::sign_message(self, digest).await
    }
}

/// Optional structured classification for typed signer errors.
///
/// Implementations expose only the deterministic, non-sensitive
/// EIP-1193 provider error code so the surrounding signing crate can
/// route a method-aware rejection label to downstream consumers
/// without coupling to backend-specific error representations or to
/// the upstream error's `Display` shape. Returning `None` from every
/// variant — the default — lets the caller fall back to its existing
/// redacted `Display` path so unrelated signer failures continue to
/// redact in full.
///
/// The single exposed code carries the standardised EIP-1193
/// numeric provider error class (`4001` for user rejections, `4100`
/// for unauthorised, `4900` for disconnected, etc.). EIP-1193
/// defines these codes as public protocol classifications, so the
/// trait deliberately exposes the numeric code without any
/// implementer-controlled wallet message.
///
/// Implementers should match only on their own typed rejection
/// variants and return the code already carried in the variant's
/// `code` field — they should not parse strings, perform string
/// matching against `Display`, or wrap arbitrary failures as
/// rejections. The contract is checked by per-crate
/// `signer_error_trait_contract` tests that pin the value returned
/// for every variant the implementer carries.
pub trait SignerError {
    /// Returns the EIP-1193 provider error code when this error
    /// represents a user-driven rejection of the signing request,
    /// or `None` for every other class of failure.
    ///
    /// The default returns `None` so an implementer can adopt the
    /// trait without immediately enumerating every variant; the
    /// caller treats `None` as "fall back to the redacted `Display`
    /// path".
    #[must_use]
    fn user_rejection_code(&self) -> Option<i32> {
        None
    }
}

/// Courtesy implementation for the canonical test-signer `Error`
/// type. Production signer error types should opt in with their own
/// typed `match` over the variants that represent EIP-1193 rejections
/// rather than rely on this passthrough.
impl SignerError for String {}

/// Courtesy implementation for borrowed message errors so tests and
/// signing helpers that accept `&str` keep the same default
/// classification posture as owned `String` errors.
impl SignerError for &str {}

/// Courtesy implementation for the never-error case so signers that
/// cannot fail are still callable through the signing helpers without
/// adding a redundant per-test impl.
impl SignerError for core::convert::Infallible {}
