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
#[allow(async_fn_in_trait)]
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
#[allow(async_fn_in_trait)]
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
#[allow(async_fn_in_trait)]
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
#[allow(async_fn_in_trait)]
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
#[allow(async_fn_in_trait)]
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
