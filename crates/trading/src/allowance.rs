use cow_sdk_contracts::{ContractId, Registry};
use cow_sdk_core::{
    Address, Amount, AsyncProvider, AsyncSigner, ContractCall, Provider, Signer, SupportedChainId,
    TransactionHash, TransactionRequest,
};

use crate::{ApprovalParameters, TradingError};

/// Resolves the canonical vault-relayer address for allowance checks.
///
/// # Panics
///
/// Panics only if the embedded deployment registry is missing the canonical
/// vault-relayer entry for a supported chain/environment pair.
fn resolve_vault_relayer(chain_id: SupportedChainId, env: cow_sdk_core::CowEnv) -> Address {
    // SAFETY: Registry::default parses the build-validated embedded manifest,
    // which must include vault-relayer addresses for supported chains.
    Registry::default()
        .address(ContractId::VaultRelayer, chain_id, env)
        .expect("canonical vault-relayer address is registered for every supported chain/env")
}

const ERC20_ALLOWANCE_ABI_JSON: &str = r#"[{"type":"function","name":"allowance","inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#;
const ERC20_APPROVE_SIGNATURE: &str = "approve(address,uint256)";

/// Reads the `CoW` Protocol vault-relayer allowance using a sync provider.
///
/// # Errors
///
/// Returns [`TradingError`] when the contract call cannot be encoded, the
/// provider read fails, or the returned allowance cannot be decoded into an
/// [`Amount`].
pub fn get_cow_protocol_allowance<P>(
    provider: &P,
    token_address: &Address,
    owner: &Address,
    chain_id: SupportedChainId,
    env: cow_sdk_core::CowEnv,
    vault_relayer_override: Option<&Address>,
) -> Result<Amount, TradingError>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    let spender = vault_relayer_override
        .cloned()
        .unwrap_or_else(|| resolve_vault_relayer(chain_id, env));
    let args_json =
        serde_json::to_string(&(owner.as_str(), spender.as_str())).map_err(|error| {
            TradingError::Contracts(cow_sdk_contracts::ContractsError::Serialization(error))
        })?;
    let raw = provider
        .read_contract(&ContractCall::new(
            token_address.clone(),
            "allowance".to_owned(),
            ERC20_ALLOWANCE_ABI_JSON.to_owned(),
            args_json,
        ))
        .map_err(|error| TradingError::Provider {
            operation: "read_contract",
            message: error.to_string(),
        })?;
    decode_allowance_result(&raw)
}

/// Reads the `CoW` Protocol vault-relayer allowance using an async provider.
///
/// # Errors
///
/// Returns [`TradingError`] when the contract call cannot be encoded, the
/// provider read fails, or the returned allowance cannot be decoded into an
/// [`Amount`].
pub async fn get_cow_protocol_allowance_async<P>(
    provider: &P,
    token_address: &Address,
    owner: &Address,
    chain_id: SupportedChainId,
    env: cow_sdk_core::CowEnv,
    vault_relayer_override: Option<&Address>,
) -> Result<Amount, TradingError>
where
    P: AsyncProvider,
    P::Error: std::fmt::Display,
{
    let spender = vault_relayer_override
        .cloned()
        .unwrap_or_else(|| resolve_vault_relayer(chain_id, env));
    let args_json =
        serde_json::to_string(&(owner.as_str(), spender.as_str())).map_err(|error| {
            TradingError::Contracts(cow_sdk_contracts::ContractsError::Serialization(error))
        })?;
    let raw = provider
        .read_contract(&ContractCall::new(
            token_address.clone(),
            "allowance".to_owned(),
            ERC20_ALLOWANCE_ABI_JSON.to_owned(),
            args_json,
        ))
        .await
        .map_err(|error| TradingError::Provider {
            operation: "read_contract",
            message: error.to_string(),
        })?;
    decode_allowance_result(&raw)
}

/// Builds the ERC-20 approval transaction for the `CoW` Protocol vault relayer.
///
/// The approval amount must fit inside the ABI `uint256` range; negative values
/// and values wider than 32 bytes are rejected.
///
/// # Errors
///
/// Returns [`TradingError`] when ABI encoding fails or `amount` is outside the
/// supported `uint256` range.
pub fn approval_transaction(
    params: &ApprovalParameters,
    chain_id: SupportedChainId,
    env: cow_sdk_core::CowEnv,
) -> Result<TransactionRequest, TradingError> {
    let spender = params
        .vault_relayer_override
        .clone()
        .unwrap_or_else(|| resolve_vault_relayer(chain_id, env));
    Ok(TransactionRequest::new(
        Some(params.token_address.clone()),
        Some(cow_sdk_core::HexData::new(encode_approve_call(
            &spender,
            &params.amount,
        )?)?),
        Some(Amount::zero()),
        None,
    ))
}

/// Sends the approval transaction using a sync signer.
///
/// # Errors
///
/// Returns [`TradingError`] when transaction construction or submission fails.
pub fn approve_cow_protocol<S>(
    signer: &S,
    params: &ApprovalParameters,
    chain_id: SupportedChainId,
    env: cow_sdk_core::CowEnv,
) -> Result<TransactionHash, TradingError>
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

/// Sends the approval transaction using an async signer.
///
/// # Errors
///
/// Returns [`TradingError`] when transaction construction or submission fails.
pub async fn approve_cow_protocol_async<S>(
    signer: &S,
    params: &ApprovalParameters,
    chain_id: SupportedChainId,
    env: cow_sdk_core::CowEnv,
) -> Result<TransactionHash, TradingError>
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

fn encode_approve_call(spender: &Address, amount: &Amount) -> Result<String, TradingError> {
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

fn encode_uint_word(value: &Amount) -> Result<[u8; 32], TradingError> {
    let bytes = value.as_biguint().to_bytes_be();
    if bytes.len() > 32 {
        return Err(TradingError::NumericOverflow {
            field: "uint256",
            value: value.to_string(),
        });
    }
    let mut out = [0u8; 32];
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    Ok(out)
}

fn decode_allowance_result(raw: &str) -> Result<Amount, TradingError> {
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(serde_json::Value::String(value)) => Ok(Amount::new(value)?),
        Ok(serde_json::Value::Number(value)) => Ok(Amount::new(value.to_string())?),
        Ok(_) => Err(TradingError::InvalidInput {
            field: "allowance",
            reason: cow_sdk_core::ValidationReason::BadShape {
                details: "response must be a string or number",
            },
        }),
        Err(_) => Ok(Amount::new(raw.to_owned())?),
    }
}
