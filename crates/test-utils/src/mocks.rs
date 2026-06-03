//! Generic recording mocks and stubs against the `cow_sdk_core` traits.
//!
//! `RecordingSigner` is an `Rc<RefCell<_>>`-backed (single-threaded /
//! wasm-friendly) recorder that returns canned values and logs the calls it
//! received. `StubHttpTransport` is a no-op `HttpTransport` whose every method
//! succeeds with an empty body. `RecordingHttpTransport` records every request
//! and replays caller-supplied canned responses in order, for asserting on the
//! requests a client makes without performing I/O.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cow_sdk_core::{
    Address, Amount, Hash32, HttpTransport, Signer, TransactionBroadcast, TransactionRequest,
    TransportError, TypedDataDomain, TypedDataField,
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
#[derive(Clone, Debug)]
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

/// A no-op [`HttpTransport`] whose every method succeeds with an empty body.
///
/// Builder-typestate tests inject this to satisfy the transport precondition
/// without performing real I/O.
#[derive(Debug, Default)]
pub struct StubHttpTransport;

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl HttpTransport for StubHttpTransport {
    async fn get(
        &self,
        _path: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        Ok(String::new())
    }
    async fn post(
        &self,
        _path: &str,
        _body: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        Ok(String::new())
    }
    async fn put(
        &self,
        _path: &str,
        _body: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        Ok(String::new())
    }
    async fn delete(
        &self,
        _path: &str,
        _body: &str,
        _headers: &[(String, String)],
        _timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        Ok(String::new())
    }
}

/// A request captured by [`RecordingHttpTransport`].
#[derive(Debug, Clone)]
pub struct RecordedRequest {
    /// The HTTP method (`"GET"`, `"POST"`, `"PUT"`, or `"DELETE"`).
    pub method: &'static str,
    /// The request path or URL.
    pub url: String,
    /// The request body (empty for `GET`).
    pub body: String,
    /// Whether the caller supplied a per-request timeout.
    pub has_timeout: bool,
}

/// A canned response for [`RecordingHttpTransport`] to replay, one per call.
#[derive(Debug, Clone)]
pub enum Canned {
    /// A success body returned through `Ok`.
    Ok(String),
    /// A non-success HTTP status surfaced as [`TransportError::HttpStatus`].
    HttpStatus {
        /// The HTTP status code.
        status: u16,
        /// The response headers.
        headers: Vec<(String, String)>,
        /// The response body.
        body: String,
    },
}

impl Canned {
    fn into_result(self) -> Result<String, TransportError> {
        match self {
            Self::Ok(body) => Ok(body),
            Self::HttpStatus {
                status,
                headers,
                body,
            } => Err(TransportError::HttpStatus {
                status,
                headers: headers
                    .into_iter()
                    .map(|(name, value)| (name, value.into()))
                    .collect(),
                body: body.into(),
            }),
        }
    }
}

/// An [`HttpTransport`] that records every request and replays a queue of
/// caller-supplied canned responses, one per call.
#[derive(Debug)]
pub struct RecordingHttpTransport {
    calls: Mutex<Vec<RecordedRequest>>,
    responses: Mutex<VecDeque<Canned>>,
}

impl RecordingHttpTransport {
    /// Builds a recorder behind an `Arc` with one canned response per expected call.
    #[must_use]
    pub fn new(responses: impl IntoIterator<Item = Canned>) -> Arc<Self> {
        Arc::new(Self {
            calls: Mutex::new(Vec::new()),
            responses: Mutex::new(responses.into_iter().collect()),
        })
    }

    /// Returns the requests captured so far, in order.
    #[must_use]
    pub fn observed(&self) -> Vec<RecordedRequest> {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn record(&self, request: RecordedRequest) -> Canned {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(request);
        self.responses
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .pop_front()
            .expect("recording transport must have a canned response for every call")
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl HttpTransport for RecordingHttpTransport {
    async fn get(
        &self,
        path: &str,
        _headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.record(RecordedRequest {
            method: "GET",
            url: path.to_owned(),
            body: String::new(),
            has_timeout: timeout.is_some(),
        })
        .into_result()
    }

    async fn post(
        &self,
        path: &str,
        body: &str,
        _headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.record(RecordedRequest {
            method: "POST",
            url: path.to_owned(),
            body: body.to_owned(),
            has_timeout: timeout.is_some(),
        })
        .into_result()
    }

    async fn put(
        &self,
        path: &str,
        body: &str,
        _headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.record(RecordedRequest {
            method: "PUT",
            url: path.to_owned(),
            body: body.to_owned(),
            has_timeout: timeout.is_some(),
        })
        .into_result()
    }

    async fn delete(
        &self,
        path: &str,
        body: &str,
        _headers: &[(String, String)],
        timeout: Option<Duration>,
    ) -> Result<String, TransportError> {
        self.record(RecordedRequest {
            method: "DELETE",
            url: path.to_owned(),
            body: body.to_owned(),
            has_timeout: timeout.is_some(),
        })
        .into_result()
    }
}
