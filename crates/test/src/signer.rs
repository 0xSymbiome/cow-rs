//! [`MockSigner`]: an in-memory [`Signer`] double that signs with a public
//! development key by default and records what it was asked to sign and send.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use alloy_dyn_abi::eip712::TypedData;
use alloy_primitives::eip191_hash_message;
use cow_sdk_contracts::RecoverableSignature;
use cow_sdk_core::{
    Address, Amount, Signer, SupportedChainId, TransactionBroadcast, TransactionHash,
    TransactionRequest, TypedDataPayload,
};
use k256::ecdsa::SigningKey;

use crate::{defaults, error::MockError};

/// The development key the default signer signs with: the secp256k1 scalar `1`,
/// the smallest valid private key.
///
/// This is the canonical development key used by Alloy's own `signer-local`
/// tests and by the `CoW` services backend's signature-recovery vectors — never
/// a real secret. Its address is [`defaults::address`], so the default signer's
/// signatures recover to the address it reports.
const DEV_SIGNING_KEY: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
];

/// A recording [`Signer`] double that really signs by default.
///
/// Cloning shares one backing store, so a clone handed to the SDK and a clone
/// kept for assertions observe the same recorded calls.
///
/// By default it signs EIP-712 typed data and EIP-191 messages with a public
/// development key, emitting canonical recoverable signatures (legacy `v` in
/// `{27, 28}`, per ADR 0022), so a signed order recovers to the reported
/// address and clears the SDK's owner-recovery gate end to end. Setting
/// [`MockSignerBuilder::address`] to a different address models a wallet that
/// reports one identity but signs with another: the recovered signer then
/// disagrees with the declared owner and posting fails closed.
/// [`MockSignerBuilder::typed_data_signature`] and
/// [`MockSignerBuilder::message_signature`] override the produced signatures
/// with fixed values for error-path and wire-shape tests.
#[derive(Clone, Debug)]
pub struct MockSigner {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Debug)]
struct Inner {
    address: Address,
    message_signature: Option<String>,
    typed_data_signature: Option<String>,
    transaction_hash: TransactionHash,
    estimated_gas: Amount,
    fail_send: Option<String>,
    fail_estimate_gas: Option<String>,
    chain_id: Option<SupportedChainId>,
    calls: SignerCalls,
}

/// A snapshot of what a [`MockSigner`] was asked to do.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
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
    /// A signer that signs with the default development key.
    #[must_use]
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Starts a builder to configure the reported address, fixed signature
    /// overrides, and injected failures.
    #[must_use]
    pub fn builder() -> MockSignerBuilder {
        MockSignerBuilder::default()
    }

    /// The address this signer reports.
    #[must_use]
    pub fn address(&self) -> Address {
        self.lock().address
    }

    /// A snapshot of the calls recorded so far.
    ///
    /// Every request the double received is recorded regardless of the response:
    /// a canned success and an injected failure both leave the request in the
    /// log, so an error-path test can assert the call was attempted.
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
    message_signature: Option<String>,
    typed_data_signature: Option<String>,
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
            message_signature: None,
            typed_data_signature: None,
            transaction_hash: defaults::transaction_hash(),
            estimated_gas: Amount::from(50_000_u64),
            fail_send: None,
            fail_estimate_gas: None,
            chain_id: None,
        }
    }
}

impl MockSignerBuilder {
    /// Sets the address the signer reports.
    ///
    /// The signer always signs with the development key, so setting this to an
    /// address other than [`defaults::address`] models a signer that reports
    /// one identity but signs with another: a posting flow then fails closed at
    /// the owner-recovery gate because the recovered signer disagrees with the
    /// declared owner.
    #[must_use]
    pub const fn address(mut self, address: Address) -> Self {
        self.address = address;
        self
    }

    /// Overrides [`Signer::sign_message`] with a fixed signature instead of
    /// really signing, for error-path and wire-shape tests.
    #[must_use]
    pub fn message_signature(mut self, signature: impl Into<String>) -> Self {
        self.message_signature = Some(signature.into());
        self
    }

    /// Overrides [`Signer::sign_typed_data_payload`] with a fixed signature
    /// instead of really signing, for error-path and wire-shape tests.
    #[must_use]
    pub fn typed_data_signature(mut self, signature: impl Into<String>) -> Self {
        self.typed_data_signature = Some(signature.into());
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
        let canned = {
            let mut guard = self.lock();
            guard.calls.signed_messages.push(message.to_vec());
            guard.message_signature.clone()
        };
        if let Some(signature) = canned {
            return Ok(signature);
        }
        sign_personal_message(message)
    }

    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        let canned = {
            let mut guard = self.lock();
            guard.calls.typed_data_payloads.push(payload.clone());
            guard.typed_data_signature.clone()
        };
        if let Some(signature) = canned {
            return Ok(signature);
        }
        sign_typed_data(payload)
    }

    async fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        let mut guard = self.lock();
        guard.calls.sent_transactions.push(tx.clone());
        if let Some(error) = &guard.fail_send {
            return Err(MockError::new(error.clone()));
        }
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

/// Signs the EIP-712 digest of `payload` with the development key.
fn sign_typed_data(payload: &TypedDataPayload) -> Result<String, MockError> {
    sign_prehash(&typed_data_digest(payload)?)
}

/// Signs `message` as an EIP-191 personal-sign message with the development key.
fn sign_personal_message(message: &[u8]) -> Result<String, MockError> {
    sign_prehash(&eip191_hash_message(message).0)
}

/// The EIP-712 signing hash of `payload`, via the canonical Alloy typed-data
/// shape — byte-identical to the SDK's own order digest.
fn typed_data_digest(payload: &TypedDataPayload) -> Result<[u8; 32], MockError> {
    let message: serde_json::Value = serde_json::from_str(payload.message_json())
        .map_err(|error| MockError::new(format!("typed-data message: {error}")))?;
    let typed: TypedData = serde_json::from_value(serde_json::json!({
        "domain": payload.domain,
        "types": payload.types,
        "primaryType": payload.primary_type,
        "message": message,
    }))
    .map_err(|error| MockError::new(format!("typed-data shape: {error}")))?;
    Ok(typed
        .eip712_signing_hash()
        .map_err(|error| MockError::new(error.to_string()))?
        .0)
}

/// secp256k1-signs `digest` with the development key and emits the canonical
/// 65-byte recoverable form through [`RecoverableSignature`] (legacy `v` in
/// `{27, 28}`), so the signature recovers under the SDK's owner-recovery gate.
fn sign_prehash(digest: &[u8; 32]) -> Result<String, MockError> {
    let key = SigningKey::from_slice(&DEV_SIGNING_KEY)
        .map_err(|error| MockError::new(error.to_string()))?;
    let (signature, recovery) = key
        .sign_prehash_recoverable(digest)
        .map_err(|error| MockError::new(error.to_string()))?;
    let mut bytes = [0u8; 65];
    bytes[..64].copy_from_slice(&signature.to_bytes());
    bytes[64] = 27 + recovery.to_byte();
    RecoverableSignature::parse_bytes(&bytes)
        .map(|signature| signature.to_hex_string())
        .map_err(|error| MockError::new(error.to_string()))
}
