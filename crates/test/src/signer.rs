//! [`MockSigner`]: an in-memory [`Signer`] double that returns canned values and
//! records what it was asked to sign and send.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use cow_sdk_core::{
    Address, Amount, Signer, SupportedChainId, TransactionBroadcast, TransactionHash,
    TransactionRequest, TypedDataPayload,
};

use crate::{defaults, error::MockError};

/// A recording, canned-response [`Signer`] double.
///
/// Cloning shares one backing store, so a clone handed to the SDK and a clone
/// kept for assertions observe the same recorded calls.
#[derive(Clone, Debug)]
pub struct MockSigner {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Debug)]
struct Inner {
    address: Address,
    message_signature: String,
    typed_data_signature: String,
    transaction_signature: String,
    transaction_hash: TransactionHash,
    estimated_gas: Amount,
    fail_send: Option<String>,
    fail_estimate_gas: Option<String>,
    chain_id: Option<SupportedChainId>,
    calls: SignerCalls,
}

/// A snapshot of what a [`MockSigner`] was asked to do.
#[derive(Clone, Debug, Default)]
pub struct SignerCalls {
    /// Transactions passed to [`Signer::send_transaction`].
    pub sent_transactions: Vec<TransactionRequest>,
    /// Messages passed to [`Signer::sign_message`].
    pub signed_messages: Vec<Vec<u8>>,
    /// Typed-data payloads passed to [`Signer::sign_typed_data_payload`],
    /// each carrying the primary-type name and the canonical message JSON.
    pub typed_data_payloads: Vec<TypedDataPayload>,
}

impl MockSigner {
    /// A signer with the canned defaults from [`crate::defaults`].
    #[must_use]
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Starts a builder to configure canned values and injected failures.
    #[must_use]
    pub fn builder() -> MockSignerBuilder {
        MockSignerBuilder::default()
    }

    /// The configured signer address.
    #[must_use]
    pub fn address(&self) -> Address {
        self.lock().address
    }

    /// A snapshot of the calls recorded so far.
    #[must_use]
    pub fn recorded(&self) -> SignerCalls {
        self.lock().calls.clone()
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for MockSigner {
    fn default() -> Self {
        Self::new()
    }
}

/// Consuming builder for [`MockSigner`].
#[derive(Clone, Debug)]
pub struct MockSignerBuilder {
    address: Address,
    message_signature: String,
    typed_data_signature: String,
    transaction_signature: String,
    transaction_hash: TransactionHash,
    estimated_gas: Amount,
    fail_send: Option<String>,
    fail_estimate_gas: Option<String>,
    chain_id: Option<SupportedChainId>,
}

impl Default for MockSignerBuilder {
    fn default() -> Self {
        Self {
            address: defaults::address(),
            message_signature: defaults::message_signature(),
            typed_data_signature: defaults::typed_data_signature(),
            transaction_signature: defaults::transaction_signature(),
            transaction_hash: defaults::transaction_hash(),
            estimated_gas: Amount::from(50_000_u64),
            fail_send: None,
            fail_estimate_gas: None,
            chain_id: None,
        }
    }
}

impl MockSignerBuilder {
    /// Sets the signer address.
    #[must_use]
    pub const fn address(mut self, address: Address) -> Self {
        self.address = address;
        self
    }

    /// Sets the value [`Signer::sign_message`] returns.
    #[must_use]
    pub fn message_signature(mut self, signature: impl Into<String>) -> Self {
        self.message_signature = signature.into();
        self
    }

    /// Sets the value [`Signer::sign_typed_data_payload`] returns.
    #[must_use]
    pub fn typed_data_signature(mut self, signature: impl Into<String>) -> Self {
        self.typed_data_signature = signature.into();
        self
    }

    /// Sets the value [`Signer::sign_transaction`] returns.
    #[must_use]
    pub fn transaction_signature(mut self, signature: impl Into<String>) -> Self {
        self.transaction_signature = signature.into();
        self
    }

    /// Sets the hash [`Signer::send_transaction`] reports.
    #[must_use]
    pub const fn transaction_hash(mut self, hash: TransactionHash) -> Self {
        self.transaction_hash = hash;
        self
    }

    /// Sets the gas [`Signer::estimate_gas`] returns.
    #[must_use]
    pub const fn estimated_gas(mut self, gas: Amount) -> Self {
        self.estimated_gas = gas;
        self
    }

    /// Makes [`Signer::send_transaction`] fail with `error`.
    #[must_use]
    pub fn fail_send(mut self, error: impl Into<String>) -> Self {
        self.fail_send = Some(error.into());
        self
    }

    /// Makes [`Signer::estimate_gas`] fail with `error`.
    #[must_use]
    pub fn fail_estimate_gas(mut self, error: impl Into<String>) -> Self {
        self.fail_estimate_gas = Some(error.into());
        self
    }

    /// Sets the statically-known chain reported through [`Signer::chain_id`].
    ///
    /// Defaults to `None` (the signer opts out of the trading chain-coherence
    /// gate); set it to model a signer bound to a specific chain.
    #[must_use]
    pub const fn chain_id(mut self, chain_id: SupportedChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Builds the signer.
    #[must_use]
    pub fn build(self) -> MockSigner {
        MockSigner {
            inner: Arc::new(Mutex::new(Inner {
                address: self.address,
                message_signature: self.message_signature,
                typed_data_signature: self.typed_data_signature,
                transaction_signature: self.transaction_signature,
                transaction_hash: self.transaction_hash,
                estimated_gas: self.estimated_gas,
                fail_send: self.fail_send,
                fail_estimate_gas: self.fail_estimate_gas,
                chain_id: self.chain_id,
                calls: SignerCalls::default(),
            })),
        }
    }
}

impl Signer for MockSigner {
    type Error = MockError;

    fn chain_id(&self) -> Option<SupportedChainId> {
        self.lock().chain_id
    }

    async fn address(&self) -> Result<Address, Self::Error> {
        Ok(self.lock().address)
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        let mut guard = self.lock();
        guard.calls.signed_messages.push(message.to_vec());
        Ok(guard.message_signature.clone())
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok(self.lock().transaction_signature.clone())
    }

    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        let mut guard = self.lock();
        guard.calls.typed_data_payloads.push(payload.clone());
        Ok(guard.typed_data_signature.clone())
    }

    async fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        let mut guard = self.lock();
        if let Some(error) = &guard.fail_send {
            return Err(MockError::new(error.clone()));
        }
        guard.calls.sent_transactions.push(tx.clone());
        Ok(TransactionBroadcast::new(guard.transaction_hash))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        let guard = self.lock();
        if let Some(error) = &guard.fail_estimate_gas {
            return Err(MockError::new(error.clone()));
        }
        Ok(guard.estimated_gas)
    }
}
