use crate::types::{Address, Amount};

use super::transaction::{TransactionBroadcast, TransactionRequest};
use super::typed_data::{TypedDataDomain, TypedDataField, TypedDataPayload};
/// Synchronous signing boundary for native or test signers.
///
/// This is an active SDK contract: signing and trading workflows accept it
/// directly, and any implementor also gets `AsyncSigner` through the blanket
/// implementation below.
pub trait Signer {
    /// Provider type that can be attached to this signer.
    type Provider;
    /// Error type returned by signer operations.
    type Error;

    /// Attaches a provider or provider-like runtime to the signer.
    fn connect(&mut self, provider: Self::Provider);
    /// Returns the signer address.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when address resolution fails.
    fn get_address(&self) -> Result<Address, Self::Error>;
    /// Signs arbitrary bytes according to the backend's message-signing rules.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error>;
    /// Signs a transaction payload.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error>;
    /// Signs an explicit typed-data payload.
    ///
    /// # Errors
    ///
    /// Returns any error from [`Signer::sign_typed_data`].
    fn sign_typed_data_payload(&self, payload: &TypedDataPayload) -> Result<String, Self::Error> {
        self.sign_typed_data(
            &payload.domain,
            payload.primary_type_fields().unwrap_or_default(),
            payload.message_json(),
        )
    }
    /// Signs typed-data components using the compatibility field-based contract.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
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
    fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error>;
    /// Estimates gas for a transaction request.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when estimation fails.
    fn estimate_gas(&self, tx: &TransactionRequest) -> Result<Amount, Self::Error>;
}

/// Asynchronous owner-address capability.
///
/// This narrow trait lets async flows ask only for signer ownership when no
/// signing operation is required.
#[expect(
    async_fn_in_trait,
    reason = "the trait surface adopts native async fn in trait per ADR 0010 runtime-neutral posture; the resulting non-Send futures are covered by the workspace future_not_send allow so wasm callbacks can satisfy the same trait without an explicit Send bound"
)]
pub trait AsyncOwner {
    /// Error type returned by owner resolution.
    type Error;

    /// Returns the signer address.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when address resolution fails.
    async fn get_address(&self) -> Result<Address, Self::Error>;
}

/// Asynchronous EIP-712 typed-data signing capability.
#[expect(
    async_fn_in_trait,
    reason = "the trait surface adopts native async fn in trait per ADR 0010 runtime-neutral posture; the resulting non-Send futures are covered by the workspace future_not_send allow so wasm callbacks can satisfy the same trait without an explicit Send bound"
)]
pub trait AsyncTypedDataSigner {
    /// Error type returned by typed-data signing.
    type Error;

    /// Signs an explicit typed-data payload.
    ///
    /// # Errors
    ///
    /// Returns any error from [`AsyncTypedDataSigner::sign_typed_data`].
    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        self.sign_typed_data(
            &payload.domain,
            payload.primary_type_fields().unwrap_or_default(),
            payload.message_json(),
        )
        .await
    }

    /// Signs typed-data components using the compatibility field-based contract.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error>;
}

/// Asynchronous digest-signing capability.
#[expect(
    async_fn_in_trait,
    reason = "the trait surface adopts native async fn in trait per ADR 0010 runtime-neutral posture; the resulting non-Send futures are covered by the workspace future_not_send allow so wasm callbacks can satisfy the same trait without an explicit Send bound"
)]
pub trait AsyncDigestSigner {
    /// Error type returned by digest signing.
    type Error;

    /// Signs raw digest bytes according to the backend's message-signing rules.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    async fn sign_digest(&self, digest: &[u8]) -> Result<String, Self::Error>;
}

/// Asynchronous EIP-1193 request capability.
#[expect(
    async_fn_in_trait,
    reason = "the trait surface adopts native async fn in trait per ADR 0010 runtime-neutral posture; the resulting non-Send futures are covered by the workspace future_not_send allow so wasm callbacks can satisfy the same trait without an explicit Send bound"
)]
pub trait AsyncEip1193 {
    /// Error type returned by provider requests.
    type Error;

    /// Executes an EIP-1193 request with string parameters.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the request fails.
    async fn request(&self, method: &str, params: &[String]) -> Result<String, Self::Error>;
}

/// Asynchronous signing boundary for wallets and async runtimes.
///
/// Synchronous signers implement this trait through the blanket implementation
/// so native trading flows can keep one async-first internal path. Narrow async
/// capability traits above are preferred for callback-shaped adapters that only
/// expose one signing operation.
#[expect(
    async_fn_in_trait,
    reason = "the trait surface adopts native async fn in trait per ADR 0010 runtime-neutral posture; the resulting non-Send futures are covered by the workspace future_not_send allow so wasm callbacks can satisfy the same trait without an explicit Send bound"
)]
pub trait AsyncSigner {
    /// Error type returned by signer operations.
    type Error;

    /// Returns the signer address.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when address resolution fails.
    async fn get_address(&self) -> Result<Address, Self::Error>;
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
    /// Signs an explicit typed-data payload.
    ///
    /// # Errors
    ///
    /// Returns any error from [`AsyncSigner::sign_typed_data`].
    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        self.sign_typed_data(
            &payload.domain,
            payload.primary_type_fields().unwrap_or_default(),
            payload.message_json(),
        )
        .await
    }
    /// Signs typed-data components using the compatibility field-based contract.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error>;
    /// Sends a transaction and returns the broadcast transaction hash.
    ///
    /// This confirms only that the signer backend returned a transaction hash.
    /// Use [`crate::AsyncProvider::get_transaction_receipt`] or a higher-level
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

impl<T> AsyncSigner for T
where
    T: Signer,
{
    type Error = T::Error;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Signer::get_address(self)
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        Signer::sign_message(self, message)
    }

    async fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error> {
        Signer::sign_transaction(self, tx)
    }

    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error> {
        Signer::sign_typed_data(self, domain, fields, value_json)
    }

    async fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Signer::send_transaction(self, tx)
    }

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Signer::estimate_gas(self, tx)
    }
}

impl<T> AsyncOwner for T
where
    T: AsyncSigner,
{
    type Error = T::Error;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        AsyncSigner::get_address(self).await
    }
}

impl<T> AsyncTypedDataSigner for T
where
    T: AsyncSigner,
{
    type Error = T::Error;

    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        AsyncSigner::sign_typed_data_payload(self, payload).await
    }

    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error> {
        AsyncSigner::sign_typed_data(self, domain, fields, value_json).await
    }
}

impl<T> AsyncDigestSigner for T
where
    T: AsyncSigner,
{
    type Error = T::Error;

    async fn sign_digest(&self, digest: &[u8]) -> Result<String, Self::Error> {
        AsyncSigner::sign_message(self, digest).await
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
