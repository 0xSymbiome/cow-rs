use alloy_sol_types::SolCall as _;
use cow_sdk_contracts::{ContractId, IERC20, Registry};
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
        .copied()
        .unwrap_or_else(|| resolve_vault_relayer(chain_id, env));
    let args_json = serde_json::to_string(&(owner.to_hex_string(), spender.to_hex_string()))
        .map_err(|error| {
            TradingError::Contracts(cow_sdk_contracts::ContractsError::Serialization(error))
        })?;
    let raw = provider
        .read_contract(&ContractCall::new(
            *token_address,
            "allowance".to_owned(),
            ERC20_ALLOWANCE_ABI_JSON.to_owned(),
            args_json,
        ))
        .map_err(|error| TradingError::Provider {
            operation: "read_contract",
            message: error.to_string().into(),
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
        .copied()
        .unwrap_or_else(|| resolve_vault_relayer(chain_id, env));
    let args_json = serde_json::to_string(&(owner.to_hex_string(), spender.to_hex_string()))
        .map_err(|error| {
            TradingError::Contracts(cow_sdk_contracts::ContractsError::Serialization(error))
        })?;
    let raw = provider
        .read_contract(&ContractCall::new(
            *token_address,
            "allowance".to_owned(),
            ERC20_ALLOWANCE_ABI_JSON.to_owned(),
            args_json,
        ))
        .await
        .map_err(|error| TradingError::Provider {
            operation: "read_contract",
            message: error.to_string().into(),
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
        .unwrap_or_else(|| resolve_vault_relayer(chain_id, env));
    Ok(TransactionRequest::new(
        Some(params.token_address),
        Some(cow_sdk_core::HexData::new(encode_approve_call(
            &spender,
            &params.amount,
        ))?),
        Some(Amount::ZERO),
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
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let tx = approval_transaction(params, chain_id, env)?;
    signer
        .send_transaction(&tx)
        .map(|broadcast| broadcast.transaction_hash)
        .map_err(|error| TradingError::Signer {
            operation: "send_transaction",
            message: error.to_string().into(),
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
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let tx = approval_transaction(params, chain_id, env)?;
    signer
        .send_transaction(&tx)
        .await
        .map(|broadcast| broadcast.transaction_hash)
        .map_err(|error| TradingError::Signer {
            operation: "send_transaction",
            message: error.to_string().into(),
        })
}

fn encode_approve_call(spender: &Address, amount: &Amount) -> String {
    // Routes through the workspace `alloy::sol!`-generated
    // `IERC20::approveCall` binding per ADR 0012. The selector, the
    // address word, and the uint256 word are emitted at compile time
    // through `SolCall::abi_encode`; the wire bytes are pinned
    // byte-for-byte by the parity fixture exercised at
    // `crates/contracts/tests/parity_contract.rs::assert_erc20_approve_calldata`.
    let call = IERC20::approveCall {
        spender: (*spender).into(),
        value: *amount.as_u256(),
    };
    format!("0x{}", hex::encode(call.abi_encode()))
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
        Err(_) => Ok(Amount::new(raw)?),
    }
}
