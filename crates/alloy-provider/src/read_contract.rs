//! `read_contract` ABI encode, dispatch, decode, and JSON conversion.

use alloy_dyn_abi::{DynSolType, DynSolValue, FunctionExt, JsonAbiExt};
use alloy_json_abi::{Function, JsonAbi, Param};
use alloy_network::{Ethereum, TransactionBuilder};
use alloy_primitives::{Address as AlloyAddress, B256, Bytes, I256, U256};
use alloy_provider::{DynProvider, Provider};
use cow_sdk_core::{Address, ContractCall};
use serde_json::Value;

use crate::{conversion::decode_0x_hex, error::AsyncProviderError};

/// Executes the canonical read-contract algorithm through an Alloy provider.
pub(crate) async fn execute_read_contract(
    provider: &DynProvider<Ethereum>,
    request: &ContractCall,
) -> Result<String, AsyncProviderError> {
    let abi = load_abi(&request.abi_json, request.method.as_str())?;
    let function = resolve_function(&abi, request.method.as_str())?;
    let args = serde_json::from_str::<Value>(&request.args_json).map_err(|error| {
        AsyncProviderError::Validation(format!(
            "args_json for `{}` is not valid JSON: {error}",
            request.method
        ))
    })?;
    let values = json_args_to_dyn_values(&function.inputs, &args, request.method.as_str())?;
    let calldata = function.abi_encode_input(&values).map_err(|error| {
        AsyncProviderError::Validation(format!(
            "ABI-encoding inputs for `{}` failed: {error}",
            request.method
        ))
    })?;
    let to = request
        .address
        .as_str()
        .parse::<AlloyAddress>()
        .map_err(|_| {
            AsyncProviderError::Validation(format!(
                "ContractCall.address `{}` is not a valid 20-byte hex address",
                request.address.as_str()
            ))
        })?;
    let tx = alloy_rpc_types_eth::TransactionRequest::default()
        .with_to(to)
        .with_input(Bytes::from(calldata));
    let output = provider
        .call(tx)
        .await
        .map_err(AsyncProviderError::from_alloy_transport)?;
    let decoded = function
        .abi_decode_output(output.as_ref())
        .map_err(|error| {
            AsyncProviderError::Validation(format!(
                "ABI-decoding output of `{}` failed: {error}",
                request.method
            ))
        })?;
    let json = if decoded.len() == 1 {
        dyn_value_to_json(&decoded[0])
    } else {
        Value::Array(decoded.iter().map(dyn_value_to_json).collect())
    };
    serde_json::to_string(&json).map_err(|error| {
        AsyncProviderError::Internal(format!(
            "JSON-encoding read_contract output failed: {error}"
        ))
    })
}

fn load_abi(abi_json: &str, method: &str) -> Result<JsonAbi, AsyncProviderError> {
    serde_json::from_str::<JsonAbi>(abi_json).map_err(|error| {
        AsyncProviderError::Validation(format!("failed to load ABI for `{method}`: {error}"))
    })
}

fn resolve_function<'abi>(
    abi: &'abi JsonAbi,
    method: &str,
) -> Result<&'abi Function, AsyncProviderError> {
    let functions = abi.function(method).ok_or_else(|| {
        AsyncProviderError::Validation(format!("ABI has no function named `{method}`"))
    })?;
    if functions.len() > 1 {
        return Err(AsyncProviderError::Validation(format!(
            "ABI defines {} overloads for `{method}`; read_contract requires a unique function name",
            functions.len()
        )));
    }
    functions.first().ok_or_else(|| {
        AsyncProviderError::Validation(format!("ABI has no function named `{method}`"))
    })
}

fn resolve_param_type(param: &Param, method: &str) -> Result<DynSolType, AsyncProviderError> {
    DynSolType::parse(&param.selector_type()).map_err(|error| {
        AsyncProviderError::Validation(format!(
            "failed to resolve ABI type `{}` for `{method}`: {error}",
            param.ty
        ))
    })
}

fn json_args_to_dyn_values(
    inputs: &[Param],
    args: &Value,
    method: &str,
) -> Result<Vec<DynSolValue>, AsyncProviderError> {
    match args {
        Value::Array(items) => {
            if items.len() != inputs.len() {
                return Err(AsyncProviderError::Validation(format!(
                    "method `{method}`: expected {} ABI arguments, received {}",
                    inputs.len(),
                    items.len()
                )));
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
                        AsyncProviderError::Validation(format!(
                            "method `{method}`: missing ABI argument `{}`",
                            param.name
                        ))
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
        _ => Err(AsyncProviderError::Validation(format!(
            "method `{method}`: contract arguments must be a JSON array, object, or single value"
        ))),
    }
}

#[allow(
    clippy::match_wildcard_for_single_variants,
    reason = "the wildcard stays defensive against future DynSolType variants"
)]
fn json_to_dyn_value(
    ty: &DynSolType,
    value: &Value,
    method: &str,
) -> Result<DynSolValue, AsyncProviderError> {
    match ty {
        DynSolType::Address => {
            let address = value.as_str().ok_or_else(|| {
                AsyncProviderError::Validation(format!(
                    "method `{method}`: address must be a string"
                ))
            })?;
            let address = Address::new(address)?;
            let bytes = decode_0x_hex(address.as_str()).map_err(|error| {
                AsyncProviderError::Validation(format!("method `{method}`: {error}"))
            })?;
            Ok(DynSolValue::Address(AlloyAddress::from_slice(&bytes)))
        }
        DynSolType::Uint(bits) => Ok(DynSolValue::Uint(parse_u256(value, method)?, *bits)),
        DynSolType::Int(bits) => Ok(DynSolValue::Int(parse_i256(value, method)?, *bits)),
        DynSolType::Bool => value.as_bool().map(DynSolValue::Bool).ok_or_else(|| {
            AsyncProviderError::Validation(format!("method `{method}`: bool must be a boolean"))
        }),
        DynSolType::String => value
            .as_str()
            .map(|item| DynSolValue::String(item.to_owned()))
            .ok_or_else(|| {
                AsyncProviderError::Validation(format!(
                    "method `{method}`: string must be a string"
                ))
            }),
        DynSolType::Bytes => Ok(DynSolValue::Bytes(bytes_from_json(value, method)?)),
        DynSolType::FixedBytes(length) => {
            let bytes = bytes_from_json(value, method)?;
            if bytes.len() != *length {
                return Err(AsyncProviderError::Validation(format!(
                    "method `{method}`: expected {length} fixed bytes, received {}",
                    bytes.len()
                )));
            }
            let mut buffer = [0u8; 32];
            buffer[..bytes.len()].copy_from_slice(&bytes);
            Ok(DynSolValue::FixedBytes(B256::from(buffer), *length))
        }
        DynSolType::Array(inner) => {
            let items = value.as_array().ok_or_else(|| {
                AsyncProviderError::Validation(format!(
                    "method `{method}`: array argument must be a JSON array"
                ))
            })?;
            items
                .iter()
                .map(|item| json_to_dyn_value(inner, item, method))
                .collect::<Result<Vec<_>, _>>()
                .map(DynSolValue::Array)
        }
        DynSolType::FixedArray(inner, length) => {
            let items = value.as_array().ok_or_else(|| {
                AsyncProviderError::Validation(format!(
                    "method `{method}`: array argument must be a JSON array"
                ))
            })?;
            if items.len() != *length {
                return Err(AsyncProviderError::Validation(format!(
                    "method `{method}`: expected fixed array of length {length}, received {}",
                    items.len()
                )));
            }
            items
                .iter()
                .map(|item| json_to_dyn_value(inner, item, method))
                .collect::<Result<Vec<_>, _>>()
                .map(DynSolValue::FixedArray)
        }
        DynSolType::Tuple(components) => {
            let items = value.as_array().ok_or_else(|| {
                AsyncProviderError::Validation(format!(
                    "method `{method}`: tuple arguments must be represented as a JSON array"
                ))
            })?;
            if items.len() != components.len() {
                return Err(AsyncProviderError::Validation(format!(
                    "method `{method}`: expected tuple of length {}, received {}",
                    components.len(),
                    items.len()
                )));
            }
            items
                .iter()
                .zip(components)
                .map(|(item, inner)| json_to_dyn_value(inner, item, method))
                .collect::<Result<Vec<_>, _>>()
                .map(DynSolValue::Tuple)
        }
        _ => Err(AsyncProviderError::Validation(format!(
            "method `{method}`: unsupported ABI type `{ty:?}`"
        ))),
    }
}

#[allow(
    unreachable_patterns,
    reason = "alloy-dyn-abi adds CustomStruct only when its eip712 feature is unified elsewhere"
)]
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
        _ => Value::Null,
    }
}

fn bytes_from_json(value: &Value, method: &str) -> Result<Vec<u8>, AsyncProviderError> {
    match value {
        Value::String(raw) => decode_0x_hex(raw)
            .map_err(|error| AsyncProviderError::Validation(format!("method `{method}`: {error}"))),
        Value::Array(items) => items
            .iter()
            .map(|item| {
                item.as_u64()
                    .and_then(|value| u8::try_from(value).ok())
                    .ok_or_else(|| {
                        AsyncProviderError::Validation(format!(
                            "method `{method}`: byte arrays must contain u8-compatible numbers"
                        ))
                    })
            })
            .collect(),
        _ => Err(AsyncProviderError::Validation(format!(
            "method `{method}`: bytes must be a hex string or byte array"
        ))),
    }
}

fn parse_u256(value: &Value, method: &str) -> Result<U256, AsyncProviderError> {
    let raw = match value {
        Value::String(raw) => raw.clone(),
        Value::Number(number) => number.to_string(),
        _ => {
            return Err(AsyncProviderError::Validation(format!(
                "method `{method}`: numeric arguments must be strings or numbers"
            )));
        }
    };
    raw.strip_prefix("0x").map_or_else(
        || {
            U256::from_str_radix(&raw, 10).map_err(|error| {
                AsyncProviderError::Validation(format!(
                    "method `{method}`: invalid integer `{raw}`: {error}"
                ))
            })
        },
        |hex| {
            U256::from_str_radix(hex, 16).map_err(|error| {
                AsyncProviderError::Validation(format!(
                    "method `{method}`: invalid integer `{raw}`: {error}"
                ))
            })
        },
    )
}

fn parse_i256(value: &Value, method: &str) -> Result<I256, AsyncProviderError> {
    let raw = match value {
        Value::String(raw) => raw.clone(),
        Value::Number(number) => number.to_string(),
        _ => {
            return Err(AsyncProviderError::Validation(format!(
                "method `{method}`: numeric arguments must be strings or numbers"
            )));
        }
    };
    if let Some(hex) = raw.strip_prefix("0x") {
        let unsigned = U256::from_str_radix(hex, 16).map_err(|error| {
            AsyncProviderError::Validation(format!(
                "method `{method}`: invalid hexadecimal signed integer `{raw}`: {error}"
            ))
        })?;
        Ok(I256::from_raw(unsigned))
    } else {
        I256::from_dec_str(&raw).map_err(|error| {
            AsyncProviderError::Validation(format!(
                "method `{method}`: invalid signed integer `{raw}`: {error}"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_dyn_sol_values_handles_uint256_string_decimal() {
        let inputs = vec![Param {
            ty: "uint256".to_owned(),
            name: "amount".to_owned(),
            components: Vec::new(),
            internal_type: None,
        }];
        let values = json_args_to_dyn_values(&inputs, &json!(["42"]), "value").unwrap();
        assert!(matches!(values[0], DynSolValue::Uint(_, 256)));
    }

    #[test]
    fn parse_dyn_sol_values_handles_uint256_hex_prefix() {
        let inputs = vec![Param {
            ty: "uint256".to_owned(),
            name: "amount".to_owned(),
            components: Vec::new(),
            internal_type: None,
        }];
        let values = json_args_to_dyn_values(&inputs, &json!(["0x2a"]), "value").unwrap();
        assert_eq!(dyn_value_to_json(&values[0]), json!("42"));
    }

    #[test]
    fn parse_dyn_sol_values_handles_address() {
        let inputs = vec![Param {
            ty: "address".to_owned(),
            name: "owner".to_owned(),
            components: Vec::new(),
            internal_type: None,
        }];
        let values = json_args_to_dyn_values(
            &inputs,
            &json!(["0x1111111111111111111111111111111111111111"]),
            "owner",
        )
        .unwrap();
        assert_eq!(
            dyn_value_to_json(&values[0]),
            json!("0x1111111111111111111111111111111111111111")
        );
    }

    #[test]
    fn parse_dyn_sol_values_handles_tuple_recursive() {
        let inputs = vec![Param {
            ty: "(uint256,bool)".to_owned(),
            name: "tuple".to_owned(),
            components: Vec::new(),
            internal_type: None,
        }];
        let values = json_args_to_dyn_values(&inputs, &json!([[7, true]]), "tuple").unwrap();
        assert_eq!(dyn_value_to_json(&values[0]), json!(["7", true]));
    }

    #[test]
    fn resolve_function_rejects_overloads() {
        let abi: JsonAbi = serde_json::from_str(
            r#"[{"type":"function","name":"f","inputs":[{"type":"uint256","name":"a"}],"outputs":[]},{"type":"function","name":"f","inputs":[{"type":"address","name":"a"}],"outputs":[]}]"#,
        )
        .unwrap();
        assert!(matches!(
            resolve_function(&abi, "f"),
            Err(AsyncProviderError::Validation(_))
        ));
    }
}
