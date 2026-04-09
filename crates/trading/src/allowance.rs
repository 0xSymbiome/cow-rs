use num_bigint::Sign;

use cow_sdk_core::{
    Address, AsyncProvider, AsyncSigner, ContractCall, Provider, Signer, SupportedChainId,
    TransactionRequest, vault_relayer_address,
};

use crate::slippage::parse_integer;
use crate::{ApprovalParameters, TradingError};

const ERC20_ALLOWANCE_ABI_JSON: &str = r#"[{"type":"function","name":"allowance","inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#;
const ERC20_APPROVE_SIGNATURE: &str = "approve(address,uint256)";

pub fn get_cow_protocol_allowance<P>(
    provider: &P,
    token_address: &Address,
    owner: &Address,
    chain_id: SupportedChainId,
    env: cow_sdk_core::CowEnv,
    vault_relayer_override: Option<&Address>,
) -> Result<String, TradingError>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    let spender = vault_relayer_override
        .cloned()
        .unwrap_or_else(|| vault_relayer_address(chain_id, env));
    let args_json = serde_json::to_string(&(owner.as_str(), spender.as_str()))
        .map_err(|error| TradingError::InvalidInput(error.to_string()))?;
    provider
        .read_contract(&ContractCall {
            address: token_address.clone(),
            method: "allowance".to_owned(),
            abi_json: ERC20_ALLOWANCE_ABI_JSON.to_owned(),
            args_json,
        })
        .map_err(|error| TradingError::Provider {
            operation: "read_contract",
            message: error.to_string(),
        })
}

pub async fn get_cow_protocol_allowance_async<P>(
    provider: &P,
    token_address: &Address,
    owner: &Address,
    chain_id: SupportedChainId,
    env: cow_sdk_core::CowEnv,
    vault_relayer_override: Option<&Address>,
) -> Result<String, TradingError>
where
    P: AsyncProvider,
    P::Error: std::fmt::Display,
{
    let spender = vault_relayer_override
        .cloned()
        .unwrap_or_else(|| vault_relayer_address(chain_id, env));
    let args_json = serde_json::to_string(&(owner.as_str(), spender.as_str()))
        .map_err(|error| TradingError::InvalidInput(error.to_string()))?;
    provider
        .read_contract(&ContractCall {
            address: token_address.clone(),
            method: "allowance".to_owned(),
            abi_json: ERC20_ALLOWANCE_ABI_JSON.to_owned(),
            args_json,
        })
        .await
        .map_err(|error| TradingError::Provider {
            operation: "read_contract",
            message: error.to_string(),
        })
}

pub fn approval_transaction(
    params: &ApprovalParameters,
    chain_id: SupportedChainId,
    env: cow_sdk_core::CowEnv,
) -> Result<TransactionRequest, TradingError> {
    let spender = params
        .vault_relayer_address
        .clone()
        .unwrap_or_else(|| vault_relayer_address(chain_id, env));
    Ok(TransactionRequest {
        to: Some(params.token_address.clone()),
        data: Some(encode_approve_call(&spender, &params.amount)?),
        value: Some("0".to_owned()),
        gas_limit: None,
    })
}

pub fn approve_cow_protocol<S>(
    signer: &S,
    params: &ApprovalParameters,
    chain_id: SupportedChainId,
    env: cow_sdk_core::CowEnv,
) -> Result<String, TradingError>
where
    S: Signer,
    S::Error: std::fmt::Display,
{
    let tx = approval_transaction(params, chain_id, env)?;
    signer
        .send_transaction(&tx)
        .map(|receipt| receipt.transaction_hash)
        .map_err(|error| TradingError::Signer {
            operation: "send_transaction",
            message: error.to_string(),
        })
}

pub async fn approve_cow_protocol_async<S>(
    signer: &S,
    params: &ApprovalParameters,
    chain_id: SupportedChainId,
    env: cow_sdk_core::CowEnv,
) -> Result<String, TradingError>
where
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    let tx = approval_transaction(params, chain_id, env)?;
    signer
        .send_transaction(&tx)
        .await
        .map(|receipt| receipt.transaction_hash)
        .map_err(|error| TradingError::Signer {
            operation: "send_transaction",
            message: error.to_string(),
        })
}

fn encode_approve_call(spender: &Address, amount: &str) -> Result<String, TradingError> {
    let selector = cow_sdk_contracts::function_magic_value(ERC20_APPROVE_SIGNATURE);
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&decode_hex_field(&selector)?);
    encoded.extend_from_slice(&encode_address_word(spender)?);
    encoded.extend_from_slice(&encode_uint_word(amount)?);
    Ok(format!("0x{}", hex::encode(encoded)))
}

fn decode_hex_field(value: &str) -> Result<Vec<u8>, TradingError> {
    let Some(stripped) = value.strip_prefix("0x") else {
        return Err(TradingError::InvalidNumeric {
            field: "hex",
            value: value.to_owned(),
        });
    };
    hex::decode(stripped).map_err(|_| TradingError::InvalidNumeric {
        field: "hex",
        value: value.to_owned(),
    })
}

fn encode_address_word(address: &Address) -> Result<[u8; 32], TradingError> {
    let bytes = decode_hex_field(address.as_str())?;
    if bytes.len() != 20 {
        return Err(TradingError::InvalidNumeric {
            field: "address",
            value: address.as_str().to_owned(),
        });
    }
    let mut out = [0u8; 32];
    out[12..].copy_from_slice(&bytes);
    Ok(out)
}

fn encode_uint_word(value: &str) -> Result<[u8; 32], TradingError> {
    let parsed = parse_integer("uint256", value)?;
    let (sign, bytes) = parsed.to_bytes_be();
    if sign == Sign::Minus {
        return Err(TradingError::InvalidNumeric {
            field: "uint256",
            value: value.to_owned(),
        });
    }
    if bytes.len() > 32 {
        return Err(TradingError::NumericOverflow {
            field: "uint256",
            value: value.to_owned(),
        });
    }
    let mut out = [0u8; 32];
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    Ok(out)
}
