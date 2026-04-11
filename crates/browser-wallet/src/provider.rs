use std::{cell::RefCell, io::Cursor, rc::Rc};

use async_trait::async_trait;
use ethabi::{Contract, Param, ParamType, Token, ethereum_types::U256};
use num_bigint::BigUint;
use serde_json::{Map, Value, json};

use cow_sdk_core::{
    Address, Amount, AsyncProvider, BlockInfo, ChainId, ContractCall, ContractHandle, HexData,
    TransactionHash, TransactionReceipt, TransactionRequest,
};

use crate::{
    BrowserWalletError, EventLog, WalletEvent, WalletSession,
    events::{WalletRuntimeBindingHandle, update_wallet_session},
    signer::Eip1193Signer,
};

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait(?Send))]
pub trait Eip1193Transport {
    fn label(&self) -> &str;
    async fn request(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, BrowserWalletError>;

    fn attach_session_sync(
        &self,
        _session: Rc<RefCell<WalletSession>>,
        _events: EventLog,
    ) -> Option<WalletRuntimeBindingHandle> {
        None
    }
}

#[derive(Clone)]
pub struct Eip1193Provider {
    transport: Rc<dyn Eip1193Transport>,
    session: Rc<RefCell<WalletSession>>,
    events: EventLog,
    _runtime_binding: Option<WalletRuntimeBindingHandle>,
}

impl Eip1193Provider {
    pub(crate) fn new(
        transport: Rc<dyn Eip1193Transport>,
        session: Rc<RefCell<WalletSession>>,
        events: EventLog,
    ) -> Self {
        let runtime_binding = transport.attach_session_sync(session.clone(), events.clone());
        Self {
            transport,
            session,
            events,
            _runtime_binding: runtime_binding,
        }
    }

    pub fn session(&self) -> WalletSession {
        self.session.borrow().clone()
    }

    pub(crate) fn events(&self) -> EventLog {
        self.events.clone()
    }

    pub fn selected_account(&self) -> Option<Address> {
        self.session.borrow().selected_account.clone()
    }

    pub fn reset_session(&self) -> WalletSession {
        let wallet_label = self.session.borrow().wallet_label.clone();
        self.update_session(|session| {
            *session = WalletSession {
                wallet_label,
                ..WalletSession::default()
            };
        });
        self.session()
    }

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
            session.accounts = accounts.clone();
            session.selected_account = accounts.first().cloned();
        });
        Ok(accounts)
    }

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

#[allow(async_fn_in_trait)]
impl AsyncProvider for Eip1193Provider {
    type Signer = Eip1193Signer;
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
        Ok(Some(TransactionReceipt {
            transaction_hash: TransactionHash::new(hash)?,
        }))
    }

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
        let contract = load_contract(&request.abi_json, request.method.as_str())?;
        let function = contract
            .function(request.method.as_str())
            .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
        let args = serde_json::from_str::<Value>(&request.args_json)
            .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
        let tokens = json_args_to_tokens(&function.inputs, &args, request.method.as_str())?;
        let input = function
            .encode_input(&tokens)
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
            .decode_output(&bytes)
            .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
        let value = if decoded.len() == 1 {
            token_to_json(&decoded[0])
        } else {
            Value::Array(decoded.iter().map(token_to_json).collect())
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
        Ok(BlockInfo { number, hash })
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle {
            address: address.clone(),
            abi_json: abi_json.to_owned(),
        })
    }
}

pub(crate) fn hex_quantity(value: &str) -> Result<String, BrowserWalletError> {
    let parsed = if let Some(stripped) = value.strip_prefix("0x") {
        BigUint::parse_bytes(stripped.as_bytes(), 16)
    } else {
        BigUint::parse_bytes(value.as_bytes(), 10)
    }
    .ok_or_else(|| BrowserWalletError::serialization(format!("invalid quantity `{value}`")))?;

    if parsed == BigUint::default() {
        Ok("0x0".to_owned())
    } else {
        Ok(format!("0x{}", parsed.to_str_radix(16)))
    }
}

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
            Value::String(hex_quantity(value.as_str())?),
        );
    }
    if let Some(gas_limit) = &tx.gas_limit {
        object.insert(
            "gas".to_owned(),
            Value::String(hex_quantity(gas_limit.as_str())?),
        );
    }
    Ok(Value::Object(object))
}

fn load_contract(abi_json: &str, method: &str) -> Result<Contract, BrowserWalletError> {
    Contract::load(Cursor::new(abi_json.as_bytes())).map_err(|error| {
        BrowserWalletError::serialization(format!("failed to load ABI for `{method}`: {error}"))
    })
}

fn json_args_to_tokens(
    inputs: &[Param],
    args: &Value,
    method: &str,
) -> Result<Vec<Token>, BrowserWalletError> {
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
                .map(|(value, param)| json_to_token(&param.kind, value, method))
                .collect()
        }
        Value::Object(map) => {
            if inputs.len() == 1 && inputs[0].name.is_empty() {
                return Ok(vec![json_to_token(&inputs[0].kind, args, method)?]);
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
                    json_to_token(&param.kind, value, method)
                })
                .collect()
        }
        other if inputs.len() == 1 => Ok(vec![json_to_token(&inputs[0].kind, other, method)?]),
        _ => Err(BrowserWalletError::malformed_response(
            method,
            "contract arguments must be a JSON array, object, or single value",
        )),
    }
}

fn json_to_token(
    kind: &ParamType,
    value: &Value,
    method: &str,
) -> Result<Token, BrowserWalletError> {
    match kind {
        ParamType::Address => {
            let address = value.as_str().ok_or_else(|| {
                BrowserWalletError::malformed_response(method, "address must be a string")
            })?;
            let address = Address::new(address)?;
            let bytes = decode_hex(address.as_str(), method)?;
            Ok(Token::Address(ethabi::Address::from_slice(&bytes)))
        }
        ParamType::Uint(_) => Ok(Token::Uint(parse_u256(value, method)?)),
        ParamType::Int(_) => Ok(Token::Int(parse_u256(value, method)?)),
        ParamType::Bool => value.as_bool().map(Token::Bool).ok_or_else(|| {
            BrowserWalletError::malformed_response(method, "bool must be a boolean")
        }),
        ParamType::String => value
            .as_str()
            .map(|item| Token::String(item.to_owned()))
            .ok_or_else(|| {
                BrowserWalletError::malformed_response(method, "string must be a string")
            }),
        ParamType::Bytes => Ok(Token::Bytes(bytes_from_json(value, method)?)),
        ParamType::FixedBytes(length) => {
            let bytes = bytes_from_json(value, method)?;
            if bytes.len() != *length {
                return Err(BrowserWalletError::malformed_response(
                    method,
                    format!("expected {length} fixed bytes, received {}", bytes.len()),
                ));
            }
            Ok(Token::FixedBytes(bytes))
        }
        ParamType::Array(inner) => {
            let items = value.as_array().ok_or_else(|| {
                BrowserWalletError::malformed_response(
                    method,
                    "array argument must be a JSON array",
                )
            })?;
            items
                .iter()
                .map(|item| json_to_token(inner, item, method))
                .collect::<Result<Vec<_>, _>>()
                .map(Token::Array)
        }
        ParamType::FixedArray(inner, length) => {
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
                .map(|item| json_to_token(inner, item, method))
                .collect::<Result<Vec<_>, _>>()
                .map(Token::FixedArray)
        }
        ParamType::Tuple(components) => {
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
                .map(|(item, kind)| json_to_token(kind, item, method))
                .collect::<Result<Vec<_>, _>>()
                .map(Token::Tuple)
        }
    }
}

fn token_to_json(token: &Token) -> Value {
    match token {
        Token::Address(address) => Value::String(format!("0x{}", hex::encode(address.as_bytes()))),
        Token::FixedBytes(bytes) | Token::Bytes(bytes) => {
            Value::String(format!("0x{}", hex::encode(bytes)))
        }
        Token::Int(value) | Token::Uint(value) => Value::String(value.to_string()),
        Token::Bool(value) => Value::Bool(*value),
        Token::String(value) => Value::String(value.clone()),
        Token::Array(items) | Token::FixedArray(items) | Token::Tuple(items) => {
            Value::Array(items.iter().map(token_to_json).collect())
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
    let normalized = if let Some(stripped) = raw.strip_prefix("0x") {
        BigUint::parse_bytes(stripped.as_bytes(), 16)
    } else {
        BigUint::parse_bytes(raw.as_bytes(), 10)
    }
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
    Ok(U256::from_big_endian(&bytes))
}

pub(crate) fn decode_hex(value: &str, method: &str) -> Result<Vec<u8>, BrowserWalletError> {
    let stripped = value.strip_prefix("0x").ok_or_else(|| {
        BrowserWalletError::malformed_response(method, "hex value must be 0x-prefixed")
    })?;
    hex::decode(stripped)
        .map_err(|error| BrowserWalletError::malformed_response(method, error.to_string()))
}
