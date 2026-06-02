//! Generic recording mocks against the `cow_sdk_core` traits.
//!
//! `RecordingSigner` is an `Rc<RefCell<_>>`-backed (single-threaded /
//! wasm-friendly) recorder that returns canned values and logs the calls it
//! received.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use cow_sdk_core::{
    Address, Amount, Hash32, Signer, TransactionBroadcast, TransactionRequest, TypedDataDomain,
    TypedDataField,
};

/// The canonical canned broadcast hash returned by the recording mocks.
///
/// # Panics
/// Never panics — the literal is a valid 32-byte hex string.
#[must_use]
pub fn canned_tx_hash() -> Hash32 {
    Hash32::new(format!("0x{}", "fa".repeat(32))).expect("canned hash is valid")
}

/// A recorded `sign_typed_data` invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedDataCall {
    /// The domain passed to the signer.
    pub domain: TypedDataDomain,
    /// The typed-data fields passed to the signer.
    pub fields: Vec<TypedDataField>,
    /// The JSON-encoded message value.
    pub value_json: String,
}

/// The call log accumulated by a [`RecordingSigner`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordedCalls {
    /// Every `sign_message` payload, in order.
    pub messages: Vec<Vec<u8>>,
    /// Every `sign_typed_data` invocation, in order.
    pub typed_data: Vec<TypedDataCall>,
}

/// A typed signer error for the recorder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingSignerError(pub String);

impl fmt::Display for RecordingSignerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl cow_sdk_core::SignerError for RecordingSignerError {}

/// An `Rc<RefCell<_>>`-backed recording signer (single-threaded / wasm-friendly).
#[derive(Clone)]
pub struct RecordingSigner {
    /// The address returned by `get_address`.
    pub address: Address,
    /// The signature returned by `sign_typed_data`.
    pub typed_data_signature: String,
    /// The signature returned by `sign_message`.
    pub message_signature: String,
    /// The shared call log.
    pub calls: Rc<RefCell<RecordedCalls>>,
}

impl RecordingSigner {
    /// Constructs a recorder with the placeholder address `0x4444…` and
    /// deterministic canned signatures.
    ///
    /// # Panics
    /// Never panics — the placeholder address is a valid hex string.
    #[must_use]
    pub fn new() -> Self {
        Self {
            address: Address::new("0x4444444444444444444444444444444444444444")
                .expect("placeholder address is valid"),
            typed_data_signature: format!("0x{}1b", "aa".repeat(64)),
            message_signature: format!("0x{}1b", "bb".repeat(64)),
            calls: Rc::new(RefCell::new(RecordedCalls::default())),
        }
    }
}

impl Default for RecordingSigner {
    fn default() -> Self {
        Self::new()
    }
}

impl Signer for RecordingSigner {
    type Error = RecordingSignerError;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.address)
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        self.calls.borrow_mut().messages.push(message.to_vec());
        Ok(self.message_signature.clone())
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok("0xsigned-transaction".to_owned())
    }

    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error> {
        self.calls.borrow_mut().typed_data.push(TypedDataCall {
            domain: domain.clone(),
            fields: fields.to_vec(),
            value_json: value_json.to_owned(),
        });
        Ok(self.typed_data_signature.clone())
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Ok(TransactionBroadcast::new(canned_tx_hash()))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Ok(Amount::from(21_000u32))
    }
}
