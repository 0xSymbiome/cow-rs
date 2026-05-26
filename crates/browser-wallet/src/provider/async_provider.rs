use std::str::FromStr;

use alloy_dyn_abi::{DynSolType, DynSolValue, FunctionExt, JsonAbiExt};
use alloy_json_abi::{JsonAbi, Param};
use alloy_primitives::{B256, I256, U256};
use serde_json::{Map, Value, json};

use cow_sdk_core::{
    Address, Amount, AsyncProvider, BlockHash, BlockInfo, ChainId, ContractCall, ContractHandle,
    HexData, TransactionHash, TransactionReceipt, TransactionRequest, TransactionStatus,
};

use crate::BrowserWalletError;

use super::Eip1193Provider;

impl AsyncProvider for Eip1193Provider {
    type Error = BrowserWalletError;

    async fn get_chain_id(&self) -> Result<ChainId, Self::Error> {
        self.query_chain_id().await
    }

    async fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error> {
        let value = self
            .request(
                "eth_getCode",
                Some(json!([address.to_hex_string(), "latest"])),
            )
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
                Some(json!([transaction_hash.to_hex_string()])),
            )
            .await?;
        if value.is_null() {
            return Ok(None);
        }
        Ok(Some(parse_transaction_receipt(&value)?))
    }

    async fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error> {
        let value = self
            .request(
                "eth_getStorageAt",
                Some(json!([address.to_hex_string(), slot, "latest"])),
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
                    "to": request.address.to_hex_string(),
                    "data": format!("0x{}", alloy_primitives::hex::encode(input)),
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
        Ok(ContractHandle::new(*address, abi_json.to_owned()))
    }
}

pub(crate) fn hex_quantity(value: &str) -> Result<String, BrowserWalletError> {
    let parsed = value
        .strip_prefix("0x")
        .map_or_else(
            || U256::from_str_radix(value, 10),
            |stripped| U256::from_str_radix(stripped, 16),
        )
        .map_err(|_| BrowserWalletError::serialization(format!("invalid quantity `{value}`")))?;

    Ok(format!("0x{parsed:x}"))
}

pub(crate) fn parse_chain_id_value(
    value: &Value,
    method: &str,
) -> Result<ChainId, BrowserWalletError> {
    let parsed = match value {
        Value::String(raw) => raw
            .strip_prefix("0x")
            .map_or_else(
                || U256::from_str_radix(raw, 10),
                |stripped| U256::from_str_radix(stripped, 16),
            )
            .map_err(|error| BrowserWalletError::malformed_response(method, error.to_string()))?,
        Value::Number(number) => U256::from(number.as_u64().ok_or_else(|| {
            BrowserWalletError::malformed_response(method, "expected a u64-compatible number")
        })?),
        other => {
            return Err(BrowserWalletError::malformed_response(
                method,
                format!("expected string or number chain id, received {other}"),
            ));
        }
    };
    u64::try_from(parsed).map_err(|_| {
        BrowserWalletError::malformed_response(
            method,
            format!("chain id `{value}` exceeds u64 bounds"),
        )
    })
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

/// Parses an `eth_getTransactionReceipt` JSON-RPC response into the SDK
/// receipt contract.
///
/// Optional fields are tolerant of absence but strict on malformed values:
/// missing or `null` fields become `None`; present invalid fields return
/// [`BrowserWalletError::MalformedResponse`] with the field name.
fn parse_transaction_receipt(value: &Value) -> Result<TransactionReceipt, BrowserWalletError> {
    let transaction_hash_raw = value
        .get("transactionHash")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            BrowserWalletError::malformed_response(
                "eth_getTransactionReceipt",
                "receipt must include `transactionHash`",
            )
        })?;
    let transaction_hash = TransactionHash::new(transaction_hash_raw).map_err(|error| {
        BrowserWalletError::malformed_response(
            "eth_getTransactionReceipt",
            format!("transactionHash: {error}"),
        )
    })?;

    Ok(TransactionReceipt::from_parts(
        transaction_hash,
        parse_optional_status(value.get("status"))?,
        parse_optional_u64_quantity(value.get("blockNumber"), "blockNumber")?,
        parse_optional_block_hash(value.get("blockHash"))?,
        parse_optional_amount_quantity(value.get("gasUsed"), "gasUsed")?,
        parse_optional_address(value.get("from"), "from")?,
        parse_optional_address(value.get("to"), "to")?,
    ))
}

fn parse_optional_status(
    value: Option<&Value>,
) -> Result<Option<TransactionStatus>, BrowserWalletError> {
    let Some(raw) = value else {
        return Ok(None);
    };
    if raw.is_null() {
        return Ok(None);
    }
    match raw.as_str() {
        Some("0x1") => Ok(Some(TransactionStatus::Success)),
        Some("0x0") => Ok(Some(TransactionStatus::Reverted)),
        Some(other) => Err(BrowserWalletError::malformed_response(
            "eth_getTransactionReceipt",
            format!("status: unrecognized value `{other}`"),
        )),
        None => Err(BrowserWalletError::malformed_response(
            "eth_getTransactionReceipt",
            "status: expected hex-encoded string",
        )),
    }
}

fn parse_optional_u64_quantity(
    value: Option<&Value>,
    field: &'static str,
) -> Result<Option<u64>, BrowserWalletError> {
    let Some(raw) = value else {
        return Ok(None);
    };
    if raw.is_null() {
        return Ok(None);
    }
    let raw = raw.as_str().ok_or_else(|| {
        BrowserWalletError::malformed_response(
            "eth_getTransactionReceipt",
            format!("{field}: expected hex-encoded string"),
        )
    })?;
    let hex = raw.strip_prefix("0x").ok_or_else(|| {
        BrowserWalletError::malformed_response(
            "eth_getTransactionReceipt",
            format!("{field}: missing `0x` prefix"),
        )
    })?;
    u64::from_str_radix(hex, 16).map(Some).map_err(|error| {
        BrowserWalletError::malformed_response(
            "eth_getTransactionReceipt",
            format!("{field}: {error}"),
        )
    })
}

fn parse_optional_block_hash(
    value: Option<&Value>,
) -> Result<Option<BlockHash>, BrowserWalletError> {
    let Some(raw) = value else {
        return Ok(None);
    };
    if raw.is_null() {
        return Ok(None);
    }
    let raw = raw.as_str().ok_or_else(|| {
        BrowserWalletError::malformed_response(
            "eth_getTransactionReceipt",
            "blockHash: expected hex-encoded string",
        )
    })?;
    BlockHash::new(raw).map(Some).map_err(|error| {
        BrowserWalletError::malformed_response(
            "eth_getTransactionReceipt",
            format!("blockHash: {error}"),
        )
    })
}

fn parse_optional_amount_quantity(
    value: Option<&Value>,
    field: &'static str,
) -> Result<Option<Amount>, BrowserWalletError> {
    let Some(raw) = value else {
        return Ok(None);
    };
    if raw.is_null() {
        return Ok(None);
    }
    let raw = raw.as_str().ok_or_else(|| {
        BrowserWalletError::malformed_response(
            "eth_getTransactionReceipt",
            format!("{field}: expected hex-encoded string"),
        )
    })?;
    Amount::new(raw).map(Some).map_err(|error| {
        BrowserWalletError::malformed_response(
            "eth_getTransactionReceipt",
            format!("{field}: {error}"),
        )
    })
}

fn parse_optional_address(
    value: Option<&Value>,
    field: &'static str,
) -> Result<Option<Address>, BrowserWalletError> {
    let Some(raw) = value else {
        return Ok(None);
    };
    if raw.is_null() {
        return Ok(None);
    }
    let raw = raw.as_str().ok_or_else(|| {
        BrowserWalletError::malformed_response(
            "eth_getTransactionReceipt",
            format!("{field}: expected hex-encoded string"),
        )
    })?;
    Address::new(raw).map(Some).map_err(|error| {
        BrowserWalletError::malformed_response(
            "eth_getTransactionReceipt",
            format!("{field}: {error}"),
        )
    })
}

fn expect_string(value: &Value, method: &str) -> Result<String, BrowserWalletError> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| BrowserWalletError::malformed_response(method, "expected string response"))
}

pub(super) fn parse_address_array(
    value: &Value,
    method: &str,
) -> Result<Vec<Address>, BrowserWalletError> {
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
        object.insert("from".to_owned(), Value::String(from.to_hex_string()));
    }
    if let Some(to) = &tx.to {
        object.insert("to".to_owned(), Value::String(to.to_hex_string()));
    }
    if let Some(data) = &tx.data {
        object.insert("data".to_owned(), Value::String(data.to_hex_string()));
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
            Ok(DynSolValue::Address(address.into_alloy()))
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
            Value::String(format!("0x{}", alloy_primitives::hex::encode(address.as_slice())))
        }
        DynSolValue::FixedBytes(word, size) => {
            Value::String(format!("0x{}", alloy_primitives::hex::encode(&word.as_slice()[..*size])))
        }
        DynSolValue::Bytes(bytes) => Value::String(format!("0x{}", alloy_primitives::hex::encode(bytes))),
        DynSolValue::Int(int, _) => Value::String(int.to_string()),
        DynSolValue::Uint(uint, _) => Value::String(uint.to_string()),
        DynSolValue::Bool(flag) => Value::Bool(*flag),
        DynSolValue::String(text) => Value::String(text.clone()),
        DynSolValue::Array(items) | DynSolValue::FixedArray(items) | DynSolValue::Tuple(items) => {
            Value::Array(items.iter().map(dyn_value_to_json).collect())
        }
        DynSolValue::CustomStruct {
            prop_names, tuple, ..
        } => Value::Object(
            prop_names
                .iter()
                .zip(tuple)
                .map(|(name, value)| (name.clone(), dyn_value_to_json(value)))
                .collect::<Map<_, _>>(),
        ),
        DynSolValue::Function(function) => {
            Value::String(format!("0x{}", alloy_primitives::hex::encode(function.as_slice())))
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
    // Delegates to `alloy_primitives::U256::from_str`, which recognises
    // both the canonical decimal and `0x`-prefixed hex forms used by the
    // JSON-RPC `eth_call` response shape and enforces the `uint256`
    // ceiling at parse time. The historical BigUint→`uint256` bound
    // check collapses into a single `from_str` call per ADR 0052.
    U256::from_str(&raw).map_err(|error| {
        BrowserWalletError::malformed_response(method, format!("invalid integer `{raw}`: {error}"))
    })
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

fn decode_hex(value: &str, method: &str) -> Result<Vec<u8>, BrowserWalletError> {
    let stripped = value.strip_prefix("0x").ok_or_else(|| {
        BrowserWalletError::malformed_response(method, "hex value must be 0x-prefixed")
    })?;
    alloy_primitives::hex::decode(stripped)
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
            Some(to),
            Some(HexData::new("0x1234").unwrap()),
            Some(Amount::new("21").unwrap()),
            Some(Amount::new("21000").unwrap()),
        );

        assert_eq!(
            transaction_to_rpc(&tx, Some(&from)).unwrap(),
            json!({
                "from": from.to_hex_string(),
                "to": to.to_hex_string(),
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
                "from": from.to_hex_string(),
            })
        );
    }
}
