//! Typed EIP-1193 provider bridge and `AsyncProvider` implementation.
//!
//! This module keeps browser-wallet request execution typed and local to the leaf crate. It does
//! not expose a generic raw wallet-RPC passthrough beyond the transport seam used by the typed
//! provider and signer adapters.

use std::{cell::RefCell, fmt, rc::Rc};

use alloy_dyn_abi::{DynSolType, DynSolValue, FunctionExt, JsonAbiExt};
use alloy_json_abi::{JsonAbi, Param};
use alloy_primitives::{Address as AlloyAddress, B256, I256, U256};
use async_trait::async_trait;
use num_bigint::BigUint;
use serde_json::{Map, Value, json};

use cow_sdk_core::{
    Address, Amount, AsyncProvider, AsyncSigningProvider, BlockInfo, ChainId, ContractCall,
    ContractHandle, HexData, Redacted, TransactionHash, TransactionReceipt, TransactionRequest,
};

use crate::{
    BrowserWalletError, EventLog, WalletEvent, WalletSession,
    events::{WalletRuntimeBindingHandle, update_wallet_session},
    signer::Eip1193Signer,
};

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait(?Send))]
/// Transport seam for typed EIP-1193 browser-wallet requests.
///
/// Implementors are responsible for method dispatch, request serialization, and optional session
/// listener attachment. The public SDK surface remains typed at the provider and signer layers,
/// while browser-runtime interop details stay private to the leaf crate.
pub trait Eip1193Transport {
    /// Returns the human-readable wallet label for session and event reporting.
    fn label(&self) -> &str;
    /// Executes one wallet request and returns the decoded JSON result.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserWalletError`] when the wallet rejects the request, reports an RPC error, or
    /// returns data that cannot be represented as JSON.
    async fn request(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, BrowserWalletError>;

    /// Optionally attaches runtime-native session listeners for provider-emitted events.
    fn attach_session_sync(
        &self,
        _session: Rc<RefCell<WalletSession>>,
        _events: EventLog,
    ) -> Option<WalletRuntimeBindingHandle> {
        None
    }
}

/// Reviewed origin label for an EIP-1193 provider binding.
///
/// The value can be a browser origin or an EIP-6963 reverse-DNS identifier.
/// Its debug and display representations are redacted; use [`Origin::as_str`]
/// only when the caller deliberately needs the raw value.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Origin(String);

impl Origin {
    /// Creates a non-empty provider origin label.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserWalletError`] when the origin is empty or contains
    /// control characters.
    pub fn new(value: impl Into<String>) -> Result<Self, BrowserWalletError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(BrowserWalletError::InvalidProviderOrigin {
                message: "provider origin must not be empty".to_owned().into(),
            });
        }
        if trimmed.chars().any(char::is_control) {
            return Err(BrowserWalletError::InvalidProviderOrigin {
                message: "provider origin must not contain control characters"
                    .to_owned()
                    .into(),
            });
        }
        if !origin_scheme_is_documented(trimmed) {
            return Err(BrowserWalletError::InvalidProviderOrigin {
                message:
                    "provider origin scheme must be http, https, test, transport, or reverse-DNS"
                        .to_owned()
                        .into(),
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Returns the raw provider origin label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn origin_scheme_is_documented(value: &str) -> bool {
    let Some((scheme, _)) = value.split_once(':') else {
        return true;
    };
    matches!(
        scheme.to_ascii_lowercase().as_str(),
        "http" | "https" | "test" | "transport"
    )
}

impl fmt::Debug for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Redacted::new(self.0.clone()).fmt(f)
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Redacted::new(self.0.clone()).fmt(f)
    }
}

/// Trust-aware builder for typed EIP-1193 providers.
///
/// Providers discovered through EIP-6963 should be built with a detected
/// origin supplied by the discovery flow. Anonymous providers must opt in
/// through [`Self::with_trusted_origin`] before construction succeeds.
pub struct Eip1193ProviderBuilder {
    transport: Rc<dyn Eip1193Transport>,
    detected_origin: Option<Origin>,
    trusted_origins: Vec<Origin>,
}

impl fmt::Debug for Eip1193ProviderBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Eip1193ProviderBuilder")
            .field("wallet_label", &self.transport.label())
            .field("detected_origin", &self.detected_origin)
            .field("trusted_origins", &self.trusted_origins)
            .finish_non_exhaustive()
    }
}

impl Eip1193ProviderBuilder {
    /// Creates a provider builder from one typed EIP-1193 transport.
    #[must_use]
    pub fn new<T>(transport: T) -> Self
    where
        T: Eip1193Transport + 'static,
    {
        Self::from_shared(Rc::new(transport))
    }

    pub(crate) fn from_shared(transport: Rc<dyn Eip1193Transport>) -> Self {
        Self {
            transport,
            detected_origin: None,
            trusted_origins: Vec::new(),
        }
    }

    pub(crate) fn with_detected_origin(mut self, origin: Origin) -> Self {
        self.detected_origin = Some(origin);
        self
    }

    /// Adds an explicitly reviewed origin for an anonymous EIP-1193 provider.
    #[must_use]
    pub fn with_trusted_origin(mut self, origin: Origin) -> Self {
        self.trusted_origins.push(origin);
        self
    }

    /// Builds a typed EIP-1193 provider.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserWalletError::UntrustedProviderOrigin`] when the
    /// provider was not discovered through EIP-6963 and no explicit trusted
    /// origin was supplied.
    pub fn build(self) -> Result<Eip1193Provider, BrowserWalletError> {
        let events = EventLog::default();
        let session = Rc::new(RefCell::new(WalletSession::new(
            false,
            None,
            Vec::new(),
            None,
            self.transport.label().to_owned(),
        )));
        self.build_with_session(session, events)
    }

    pub(crate) fn build_with_session(
        self,
        session: Rc<RefCell<WalletSession>>,
        events: EventLog,
    ) -> Result<Eip1193Provider, BrowserWalletError> {
        let origin = self.trusted_origin()?;
        {
            let mut session_state = session.borrow_mut();
            self.transport
                .label()
                .clone_into(&mut session_state.wallet_label);
        }
        Ok(Eip1193Provider::new(
            self.transport,
            session,
            events,
            origin,
        ))
    }

    fn trusted_origin(&self) -> Result<Option<Origin>, BrowserWalletError> {
        if let Some(origin) = &self.detected_origin {
            return Ok(Some(origin.clone()));
        }

        if let Some(origin) = self.trusted_origins.first() {
            warn_wallet_origin(origin, true);
            return Ok(Some(origin.clone()));
        }

        warn_anonymous_wallet_origin();
        Err(BrowserWalletError::UntrustedProviderOrigin {
            origin: Redacted::new("<anonymous>".to_owned()),
        })
    }
}

/// Typed browser-wallet provider that implements [`cow_sdk_core::AsyncProvider`]
/// and [`cow_sdk_core::AsyncSigningProvider`].
#[derive(Clone)]
pub struct Eip1193Provider {
    transport: Rc<dyn Eip1193Transport>,
    session: Rc<RefCell<WalletSession>>,
    events: EventLog,
    origin: Option<Origin>,
    _runtime_binding: Option<WalletRuntimeBindingHandle>,
}

impl fmt::Debug for Eip1193Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let session = self.session.borrow().clone();
        f.debug_struct("Eip1193Provider")
            .field("wallet_label", &session.wallet_label)
            .field("session", &session)
            .field("origin", &self.origin)
            .finish_non_exhaustive()
    }
}

impl Eip1193Provider {
    pub(crate) fn new(
        transport: Rc<dyn Eip1193Transport>,
        session: Rc<RefCell<WalletSession>>,
        events: EventLog,
        origin: Option<Origin>,
    ) -> Self {
        let runtime_binding = transport.attach_session_sync(session.clone(), events.clone());
        Self {
            transport,
            session,
            events,
            origin,
            _runtime_binding: runtime_binding,
        }
    }

    /// Returns the current normalized wallet session snapshot.
    #[must_use]
    pub fn session(&self) -> WalletSession {
        self.session.borrow().clone()
    }

    pub(crate) fn events(&self) -> EventLog {
        self.events.clone()
    }

    /// Returns the reviewed provider origin label, if one was captured at construction.
    #[must_use]
    pub const fn origin(&self) -> Option<&Origin> {
        self.origin.as_ref()
    }

    /// Returns the currently selected wallet account, when available.
    #[must_use]
    pub fn selected_account(&self) -> Option<Address> {
        self.session.borrow().selected_account.clone()
    }

    /// Clears the cached wallet session state while preserving the wallet label.
    #[must_use]
    pub fn reset_session(&self) -> WalletSession {
        let wallet_label = self.session.borrow().wallet_label.clone();
        self.update_session(move |session| {
            *session = WalletSession::new(false, None, Vec::new(), None, wallet_label);
        });
        self.session()
    }

    /// Queries wallet accounts and updates the cached session state.
    ///
    /// When `interactive` is `true`, this uses `eth_requestAccounts` and may trigger a wallet
    /// authorization prompt. When it is `false`, this uses `eth_accounts` and performs a passive
    /// account lookup only.
    ///
    /// # Errors
    ///
    /// Returns an error when the wallet rejects the request or returns a malformed account list.
    pub async fn query_accounts(
        &self,
        interactive: bool,
    ) -> Result<Vec<Address>, BrowserWalletError> {
        let method = if interactive {
            "eth_requestAccounts"
        } else {
            "eth_accounts"
        };
        let value = self.request(method, None).await?;
        let accounts = parse_address_array(&value, method)?;
        self.update_session(|session| {
            session.connected = !accounts.is_empty();
            session.accounts.clone_from(&accounts);
            session.selected_account = accounts.first().cloned();
        });
        Ok(accounts)
    }

    /// Queries the connected chain id and updates the cached session state.
    ///
    /// # Errors
    ///
    /// Returns an error when the wallet rejects `eth_chainId` or returns a malformed chain id.
    pub async fn query_chain_id(&self) -> Result<ChainId, BrowserWalletError> {
        let value = self.request("eth_chainId", None).await?;
        let chain_id = parse_chain_id_value(&value, "eth_chainId")?;
        self.update_session(|session| {
            session.chain_id = Some(chain_id);
        });
        Ok(chain_id)
    }

    pub(crate) async fn request(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, BrowserWalletError> {
        self.events.push(WalletEvent::RequestStarted {
            method: method.to_owned(),
        });
        match self.transport.request(method, params).await {
            Ok(value) => {
                self.events.push(WalletEvent::RequestSucceeded {
                    method: method.to_owned(),
                });
                Ok(value)
            }
            Err(error) => {
                self.events.push(WalletEvent::RequestFailed {
                    method: method.to_owned(),
                    message: error.to_string(),
                });
                Err(error)
            }
        }
    }

    pub(crate) fn update_session<F>(&self, updater: F)
    where
        F: FnOnce(&mut WalletSession),
    {
        update_wallet_session(&self.session, &self.events, None, updater);
    }
}

#[cfg(feature = "tracing")]
fn warn_wallet_origin(origin: &Origin, allowed: bool) {
    tracing::warn!(
        target: "cow_sdk::trust",
        origin = ?Redacted::new(origin.as_str().to_owned()),
        allowed,
        "non-discovered EIP-1193 provider origin evaluated"
    );
}

#[cfg(not(feature = "tracing"))]
fn warn_wallet_origin(_origin: &Origin, _allowed: bool) {}

#[cfg(feature = "tracing")]
fn warn_anonymous_wallet_origin() {
    tracing::warn!(
        target: "cow_sdk::trust",
        origin = ?Redacted::new("<anonymous>".to_owned()),
        allowed = false,
        "anonymous EIP-1193 provider rejected"
    );
}

#[cfg(not(feature = "tracing"))]
fn warn_anonymous_wallet_origin() {}

#[allow(async_fn_in_trait)]
impl AsyncProvider for Eip1193Provider {
    type Error = BrowserWalletError;

    async fn get_chain_id(&self) -> Result<ChainId, Self::Error> {
        self.query_chain_id().await
    }

    async fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error> {
        let value = self
            .request("eth_getCode", Some(json!([address.as_str(), "latest"])))
            .await?;
        let code = expect_string(&value, "eth_getCode")?;
        if code == "0x" || code == "0x0" {
            Ok(None)
        } else {
            Ok(Some(HexData::new(code)?))
        }
    }

    async fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        let value = self
            .request(
                "eth_getTransactionReceipt",
                Some(json!([transaction_hash.as_str()])),
            )
            .await?;
        if value.is_null() {
            return Ok(None);
        }
        let hash = value
            .get("transactionHash")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                BrowserWalletError::malformed_response(
                    "eth_getTransactionReceipt",
                    "receipt must include `transactionHash`",
                )
            })?;
        Ok(Some(TransactionReceipt::new(TransactionHash::new(hash)?)))
    }

    async fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error> {
        let value = self
            .request(
                "eth_getStorageAt",
                Some(json!([address.as_str(), slot, "latest"])),
            )
            .await?;
        HexData::new(expect_string(&value, "eth_getStorageAt")?).map_err(Into::into)
    }

    async fn call(&self, tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        let value = self
            .request(
                "eth_call",
                Some(json!([
                    transaction_to_rpc(tx, self.selected_account().as_ref())?,
                    "latest"
                ])),
            )
            .await?;
        HexData::new(expect_string(&value, "eth_call")?).map_err(Into::into)
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        let abi = load_abi(&request.abi_json, request.method.as_str())?;
        let function = resolve_function(&abi, request.method.as_str())?;
        let args = serde_json::from_str::<Value>(&request.args_json)
            .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
        let values = json_args_to_dyn_values(&function.inputs, &args, request.method.as_str())?;
        let input = function
            .abi_encode_input(&values)
            .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
        let raw = self
            .request(
                "eth_call",
                Some(json!([{
                    "to": request.address.as_str(),
                    "data": format!("0x{}", hex::encode(input)),
                }, "latest"])),
            )
            .await?;
        let raw = expect_string(&raw, "eth_call")?;
        let bytes = decode_hex(&raw, "eth_call")?;
        let decoded = function
            .abi_decode_output(&bytes)
            .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
        let value = if decoded.len() == 1 {
            dyn_value_to_json(&decoded[0])
        } else {
            Value::Array(decoded.iter().map(dyn_value_to_json).collect())
        };
        serde_json::to_string(&value)
            .map_err(|error| BrowserWalletError::serialization(error.to_string()))
    }

    async fn get_block(&self, block_tag: &str) -> Result<BlockInfo, Self::Error> {
        let value = self
            .request("eth_getBlockByNumber", Some(json!([block_tag, false])))
            .await?;
        let number = value
            .get("number")
            .ok_or_else(|| {
                BrowserWalletError::malformed_response(
                    "eth_getBlockByNumber",
                    "block response must include `number`",
                )
            })
            .and_then(|number| parse_chain_id_value(number, "eth_getBlockByNumber"))?;
        let hash = value
            .get("hash")
            .and_then(Value::as_str)
            .map(cow_sdk_core::BlockHash::new)
            .transpose()?;
        Ok(BlockInfo::new(number, hash))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(address.clone(), abi_json.to_owned()))
    }
}

#[allow(async_fn_in_trait)]
impl AsyncSigningProvider for Eip1193Provider {
    type Signer = Eip1193Signer;

    async fn create_signer(&self, signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        let account_hint = if signer_hint.trim().is_empty() {
            None
        } else {
            Some(Address::new(signer_hint.trim())?)
        };
        if let Some(expected) = &account_hint {
            let accounts = if self.session.borrow().accounts.is_empty() {
                self.query_accounts(false).await?
            } else {
                self.session.borrow().accounts.clone()
            };
            if !accounts
                .iter()
                .any(|candidate| candidate.normalized_key() == expected.normalized_key())
            {
                return Err(BrowserWalletError::malformed_response(
                    "create_signer",
                    format!("wallet does not expose account {}", expected.as_str()),
                ));
            }
        }
        Ok(Eip1193Signer::new(self.clone(), account_hint))
    }
}

pub(crate) fn hex_quantity(value: &str) -> Result<String, BrowserWalletError> {
    let parsed = value
        .strip_prefix("0x")
        .map_or_else(
            || BigUint::parse_bytes(value.as_bytes(), 10),
            |stripped| BigUint::parse_bytes(stripped.as_bytes(), 16),
        )
        .ok_or_else(|| BrowserWalletError::serialization(format!("invalid quantity `{value}`")))?;

    if parsed == BigUint::default() {
        Ok("0x0".to_owned())
    } else {
        Ok(format!("0x{}", parsed.to_str_radix(16)))
    }
}

#[allow(
    clippy::option_if_let_else,
    reason = "both hex and decimal branches wrap a multi-line map_err closure that constructs the same malformed_response error; the if let/else form keeps the two parse-radix paths visually parallel instead of nesting duplicated error construction inside two map_or_else closures"
)]
pub(crate) fn parse_chain_id_value(
    value: &Value,
    method: &str,
) -> Result<ChainId, BrowserWalletError> {
    match value {
        Value::String(raw) => {
            if let Some(stripped) = raw.strip_prefix("0x") {
                u64::from_str_radix(stripped, 16).map_err(|error| {
                    BrowserWalletError::malformed_response(method, error.to_string())
                })
            } else {
                raw.parse::<u64>().map_err(|error| {
                    BrowserWalletError::malformed_response(method, error.to_string())
                })
            }
        }
        Value::Number(number) => number.as_u64().ok_or_else(|| {
            BrowserWalletError::malformed_response(method, "expected a u64-compatible number")
        }),
        other => Err(BrowserWalletError::malformed_response(
            method,
            format!("expected string or number chain id, received {other}"),
        )),
    }
}

pub(crate) fn parse_quantity_to_decimal(
    value: &Value,
    method: &str,
) -> Result<Amount, BrowserWalletError> {
    match value {
        Value::String(raw) => Amount::new(raw.clone())
            .map_err(|error| BrowserWalletError::malformed_response(method, error.to_string())),
        _ => Err(BrowserWalletError::malformed_response(
            method,
            "expected hex quantity string",
        )),
    }
}

fn expect_string(value: &Value, method: &str) -> Result<String, BrowserWalletError> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| BrowserWalletError::malformed_response(method, "expected string response"))
}

fn parse_address_array(value: &Value, method: &str) -> Result<Vec<Address>, BrowserWalletError> {
    let items = value.as_array().ok_or_else(|| {
        BrowserWalletError::malformed_response(method, "expected an array of addresses")
    })?;
    items
        .iter()
        .map(|item| {
            item.as_str()
                .ok_or_else(|| {
                    BrowserWalletError::malformed_response(
                        method,
                        "account entries must be strings",
                    )
                })
                .and_then(|raw| Address::new(raw).map_err(Into::into))
        })
        .collect()
}

pub(crate) fn transaction_to_rpc(
    tx: &TransactionRequest,
    from: Option<&Address>,
) -> Result<Value, BrowserWalletError> {
    let mut object = Map::new();
    if let Some(from) = from {
        object.insert("from".to_owned(), Value::String(from.as_str().to_owned()));
    }
    if let Some(to) = &tx.to {
        object.insert("to".to_owned(), Value::String(to.as_str().to_owned()));
    }
    if let Some(data) = &tx.data {
        object.insert("data".to_owned(), Value::String(data.as_str().to_owned()));
    }
    if let Some(value) = &tx.value {
        object.insert(
            "value".to_owned(),
            Value::String(hex_quantity(&value.to_string())?),
        );
    }
    if let Some(gas_limit) = &tx.gas_limit {
        object.insert(
            "gas".to_owned(),
            Value::String(hex_quantity(&gas_limit.to_string())?),
        );
    }
    Ok(Value::Object(object))
}

fn load_abi(abi_json: &str, method: &str) -> Result<JsonAbi, BrowserWalletError> {
    serde_json::from_str::<JsonAbi>(abi_json).map_err(|error| {
        BrowserWalletError::serialization(format!("failed to load ABI for `{method}`: {error}"))
    })
}

fn resolve_function<'abi>(
    abi: &'abi JsonAbi,
    method: &str,
) -> Result<&'abi alloy_json_abi::Function, BrowserWalletError> {
    let functions = abi.function(method).ok_or_else(|| {
        BrowserWalletError::serialization(format!("ABI has no function named `{method}`"))
    })?;
    if functions.len() > 1 {
        return Err(BrowserWalletError::serialization(format!(
            "ABI defines {} overloads for `{method}`; typed browser-wallet bridge requires a unique function name",
            functions.len()
        )));
    }
    functions.first().ok_or_else(|| {
        BrowserWalletError::serialization(format!("ABI has no function named `{method}`"))
    })
}

fn resolve_param_type(param: &Param, method: &str) -> Result<DynSolType, BrowserWalletError> {
    DynSolType::parse(&param.selector_type()).map_err(|error| {
        BrowserWalletError::serialization(format!(
            "failed to resolve ABI type `{}` for `{method}`: {error}",
            param.ty
        ))
    })
}

fn json_args_to_dyn_values(
    inputs: &[Param],
    args: &Value,
    method: &str,
) -> Result<Vec<DynSolValue>, BrowserWalletError> {
    match args {
        Value::Array(items) => {
            if items.len() != inputs.len() {
                return Err(BrowserWalletError::malformed_response(
                    method,
                    format!(
                        "expected {} ABI arguments, received {}",
                        inputs.len(),
                        items.len()
                    ),
                ));
            }
            items
                .iter()
                .zip(inputs)
                .map(|(value, param)| {
                    let ty = resolve_param_type(param, method)?;
                    json_to_dyn_value(&ty, value, method)
                })
                .collect()
        }
        Value::Object(map) => {
            if inputs.len() == 1 && inputs[0].name.is_empty() {
                let ty = resolve_param_type(&inputs[0], method)?;
                return Ok(vec![json_to_dyn_value(&ty, args, method)?]);
            }
            inputs
                .iter()
                .map(|param| {
                    let value = map.get(&param.name).ok_or_else(|| {
                        BrowserWalletError::malformed_response(
                            method,
                            format!("missing ABI argument `{}`", param.name),
                        )
                    })?;
                    let ty = resolve_param_type(param, method)?;
                    json_to_dyn_value(&ty, value, method)
                })
                .collect()
        }
        other if inputs.len() == 1 => {
            let ty = resolve_param_type(&inputs[0], method)?;
            Ok(vec![json_to_dyn_value(&ty, other, method)?])
        }
        _ => Err(BrowserWalletError::malformed_response(
            method,
            "contract arguments must be a JSON array, object, or single value",
        )),
    }
}

#[allow(
    clippy::match_wildcard_for_single_variants,
    reason = "the wildcard stays defensive against future DynSolType variants published by the upstream alloy-dyn-abi crate"
)]
fn json_to_dyn_value(
    ty: &DynSolType,
    value: &Value,
    method: &str,
) -> Result<DynSolValue, BrowserWalletError> {
    match ty {
        DynSolType::Address => {
            let address = value.as_str().ok_or_else(|| {
                BrowserWalletError::malformed_response(method, "address must be a string")
            })?;
            let address = Address::new(address)?;
            let bytes = decode_hex(address.as_str(), method)?;
            Ok(DynSolValue::Address(AlloyAddress::from_slice(&bytes)))
        }
        DynSolType::Uint(bits) => Ok(DynSolValue::Uint(parse_u256(value, method)?, *bits)),
        DynSolType::Int(bits) => Ok(DynSolValue::Int(parse_i256(value, method)?, *bits)),
        DynSolType::Bool => value.as_bool().map(DynSolValue::Bool).ok_or_else(|| {
            BrowserWalletError::malformed_response(method, "bool must be a boolean")
        }),
        DynSolType::String => value
            .as_str()
            .map(|item| DynSolValue::String(item.to_owned()))
            .ok_or_else(|| {
                BrowserWalletError::malformed_response(method, "string must be a string")
            }),
        DynSolType::Bytes => Ok(DynSolValue::Bytes(bytes_from_json(value, method)?)),
        DynSolType::FixedBytes(length) => {
            let bytes = bytes_from_json(value, method)?;
            if bytes.len() != *length {
                return Err(BrowserWalletError::malformed_response(
                    method,
                    format!("expected {length} fixed bytes, received {}", bytes.len()),
                ));
            }
            let mut buffer = [0u8; 32];
            buffer[..bytes.len()].copy_from_slice(&bytes);
            Ok(DynSolValue::FixedBytes(B256::from(buffer), *length))
        }
        DynSolType::Array(inner) => {
            let items = value.as_array().ok_or_else(|| {
                BrowserWalletError::malformed_response(
                    method,
                    "array argument must be a JSON array",
                )
            })?;
            items
                .iter()
                .map(|item| json_to_dyn_value(inner, item, method))
                .collect::<Result<Vec<_>, _>>()
                .map(DynSolValue::Array)
        }
        DynSolType::FixedArray(inner, length) => {
            let items = value.as_array().ok_or_else(|| {
                BrowserWalletError::malformed_response(
                    method,
                    "array argument must be a JSON array",
                )
            })?;
            if items.len() != *length {
                return Err(BrowserWalletError::malformed_response(
                    method,
                    format!(
                        "expected fixed array of length {length}, received {}",
                        items.len()
                    ),
                ));
            }
            items
                .iter()
                .map(|item| json_to_dyn_value(inner, item, method))
                .collect::<Result<Vec<_>, _>>()
                .map(DynSolValue::FixedArray)
        }
        DynSolType::Tuple(components) => {
            let items = value.as_array().ok_or_else(|| {
                BrowserWalletError::malformed_response(
                    method,
                    "tuple arguments must be represented as a JSON array",
                )
            })?;
            if items.len() != components.len() {
                return Err(BrowserWalletError::malformed_response(
                    method,
                    format!(
                        "expected tuple of length {}, received {}",
                        components.len(),
                        items.len()
                    ),
                ));
            }
            items
                .iter()
                .zip(components)
                .map(|(item, inner)| json_to_dyn_value(inner, item, method))
                .collect::<Result<Vec<_>, _>>()
                .map(DynSolValue::Tuple)
        }
        _ => Err(BrowserWalletError::serialization(format!(
            "unsupported ABI type `{ty:?}` for `{method}`"
        ))),
    }
}

fn dyn_value_to_json(value: &DynSolValue) -> Value {
    match value {
        DynSolValue::Address(address) => {
            Value::String(format!("0x{}", hex::encode(address.as_slice())))
        }
        DynSolValue::FixedBytes(word, size) => {
            Value::String(format!("0x{}", hex::encode(&word.as_slice()[..*size])))
        }
        DynSolValue::Bytes(bytes) => Value::String(format!("0x{}", hex::encode(bytes))),
        DynSolValue::Int(int, _) => Value::String(int.to_string()),
        DynSolValue::Uint(uint, _) => Value::String(uint.to_string()),
        DynSolValue::Bool(flag) => Value::Bool(*flag),
        DynSolValue::String(text) => Value::String(text.clone()),
        DynSolValue::Array(items) | DynSolValue::FixedArray(items) | DynSolValue::Tuple(items) => {
            Value::Array(items.iter().map(dyn_value_to_json).collect())
        }
        DynSolValue::Function(function) => {
            Value::String(format!("0x{}", hex::encode(function.as_slice())))
        }
    }
}

fn bytes_from_json(value: &Value, method: &str) -> Result<Vec<u8>, BrowserWalletError> {
    match value {
        Value::String(raw) => decode_hex(raw, method),
        Value::Array(items) => items
            .iter()
            .map(|item| {
                item.as_u64()
                    .and_then(|value| u8::try_from(value).ok())
                    .ok_or_else(|| {
                        BrowserWalletError::malformed_response(
                            method,
                            "byte arrays must contain u8-compatible numbers",
                        )
                    })
            })
            .collect(),
        _ => Err(BrowserWalletError::malformed_response(
            method,
            "bytes must be a hex string or byte array",
        )),
    }
}

fn parse_u256(value: &Value, method: &str) -> Result<U256, BrowserWalletError> {
    let raw = match value {
        Value::String(raw) => raw.clone(),
        Value::Number(number) => number.to_string(),
        _ => {
            return Err(BrowserWalletError::malformed_response(
                method,
                "numeric arguments must be strings or numbers",
            ));
        }
    };
    let normalized = raw
        .strip_prefix("0x")
        .map_or_else(
            || BigUint::parse_bytes(raw.as_bytes(), 10),
            |stripped| BigUint::parse_bytes(stripped.as_bytes(), 16),
        )
        .ok_or_else(|| {
            BrowserWalletError::malformed_response(method, format!("invalid integer `{raw}`"))
        })?;
    let bytes = normalized.to_bytes_be();
    if bytes.len() > 32 {
        return Err(BrowserWalletError::malformed_response(
            method,
            format!("integer `{raw}` exceeds uint256 bounds"),
        ));
    }
    let mut padded = [0u8; 32];
    padded[32 - bytes.len()..].copy_from_slice(&bytes);
    Ok(U256::from_be_bytes(padded))
}

fn parse_i256(value: &Value, method: &str) -> Result<I256, BrowserWalletError> {
    let raw = match value {
        Value::String(raw) => raw.clone(),
        Value::Number(number) => number.to_string(),
        _ => {
            return Err(BrowserWalletError::malformed_response(
                method,
                "numeric arguments must be strings or numbers",
            ));
        }
    };
    if let Some(stripped) = raw.strip_prefix("0x") {
        let unsigned = U256::from_str_radix(stripped, 16).map_err(|error| {
            BrowserWalletError::malformed_response(
                method,
                format!("invalid hexadecimal signed integer `{raw}`: {error}"),
            )
        })?;
        Ok(I256::from_raw(unsigned))
    } else {
        I256::from_dec_str(&raw).map_err(|error| {
            BrowserWalletError::malformed_response(
                method,
                format!("invalid signed integer `{raw}`: {error}"),
            )
        })
    }
}

pub(crate) fn decode_hex(value: &str, method: &str) -> Result<Vec<u8>, BrowserWalletError> {
    let stripped = value.strip_prefix("0x").ok_or_else(|| {
        BrowserWalletError::malformed_response(method, "hex value must be 0x-prefixed")
    })?;
    hex::decode(stripped)
        .map_err(|error| BrowserWalletError::malformed_response(method, error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn quantity_parser_preserves_non_zero_string_values() {
        assert_eq!(
            parse_quantity_to_decimal(&json!("42"), "eth_estimateGas").unwrap(),
            Amount::new("42").unwrap()
        );
    }

    #[test]
    fn quantity_parser_rejects_non_string_values() {
        assert_eq!(
            parse_quantity_to_decimal(&json!(42), "eth_estimateGas").unwrap_err(),
            BrowserWalletError::MalformedResponse {
                method: "eth_estimateGas".to_owned().into(),
                message: "expected hex quantity string".to_owned().into(),
            }
        );
    }

    #[test]
    fn rpc_transaction_shape_keeps_present_fields_explicit_and_hex_encoded() {
        let from = Address::new("0x4444444444444444444444444444444444444444").unwrap();
        let to = Address::new("0x1111111111111111111111111111111111111111").unwrap();
        let tx = TransactionRequest::new(
            Some(to.clone()),
            Some(HexData::new("0x1234").unwrap()),
            Some(Amount::new("21").unwrap()),
            Some(Amount::new("21000").unwrap()),
        );

        assert_eq!(
            transaction_to_rpc(&tx, Some(&from)).unwrap(),
            json!({
                "from": from.as_str(),
                "to": to.as_str(),
                "data": "0x1234",
                "value": "0x15",
                "gas": "0x5208",
            })
        );
    }

    #[test]
    fn rpc_transaction_shape_omits_absent_optional_fields() {
        let from = Address::new("0x4444444444444444444444444444444444444444").unwrap();

        assert_eq!(
            transaction_to_rpc(&TransactionRequest::default(), Some(&from)).unwrap(),
            json!({
                "from": from.as_str(),
            })
        );
    }
}
