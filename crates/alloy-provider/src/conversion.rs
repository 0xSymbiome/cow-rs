//! Conversion helpers between `cow-sdk-core` values and Alloy RPC values.

use alloy_consensus::BlockHeader as _;
use alloy_network::TransactionBuilder;
use alloy_primitives::{Address as AlloyAddress, B256, U256};
use alloy_rpc_types_eth::{BlockId, BlockNumberOrTag, TransactionRequest as AlloyTransaction};
use cow_sdk_core::{
    Address, BlockInfo, HexData, TransactionHash, TransactionReceipt, TransactionRequest,
};

use crate::error::AsyncProviderError;

/// Converts a `cow-sdk-core` address into Alloy's address type.
pub(crate) fn cow_to_alloy_address(address: &Address) -> Result<AlloyAddress, AsyncProviderError> {
    address.as_str().parse::<AlloyAddress>().map_err(|_| {
        AsyncProviderError::Validation(format!("address `{}` failed alloy parse", address.as_str()))
    })
}

/// Converts a `cow-sdk-core` transaction hash into Alloy's hash type.
pub(crate) fn cow_to_alloy_hash(
    transaction_hash: &TransactionHash,
) -> Result<B256, AsyncProviderError> {
    transaction_hash.as_str().parse::<B256>().map_err(|_| {
        AsyncProviderError::Validation(format!(
            "transaction hash `{}` failed alloy parse",
            transaction_hash.as_str()
        ))
    })
}

/// Converts a core transaction request into an Alloy transaction request.
pub(crate) fn cow_request_to_alloy(
    request: &TransactionRequest,
) -> Result<AlloyTransaction, String> {
    let mut alloy_tx = AlloyTransaction::default();
    if let Some(to) = &request.to {
        alloy_tx = alloy_tx.with_to(cow_to_alloy_address(to).map_err(|error| error.to_string())?);
    }
    if let Some(data) = &request.data {
        let bytes = decode_0x_hex(data.as_str()).map_err(|error| format!("data: {error}"))?;
        alloy_tx = alloy_tx.with_input(bytes);
    }
    if let Some(value) = &request.value {
        let amount = U256::from_str_radix(&value.to_string(), 10)
            .map_err(|error| format!("value parse error: {error}"))?;
        alloy_tx = alloy_tx.with_value(amount);
    }
    if let Some(gas_limit) = &request.gas_limit {
        let gas = gas_limit
            .to_string()
            .parse::<u64>()
            .map_err(|error| format!("gas_limit parse error: {error}"))?;
        alloy_tx = alloy_tx.with_gas_limit(gas);
    }
    Ok(alloy_tx)
}

/// Converts a core block tag string into Alloy's block-id type.
pub(crate) fn cow_block_tag_to_alloy(tag: &str) -> Result<BlockId, String> {
    let normalized = tag.trim();
    let block = match normalized {
        "latest" => BlockNumberOrTag::Latest,
        "pending" => BlockNumberOrTag::Pending,
        "earliest" => BlockNumberOrTag::Earliest,
        "finalized" => BlockNumberOrTag::Finalized,
        "safe" => BlockNumberOrTag::Safe,
        value if value.starts_with("0x") && value.len() == 66 => {
            let hash = value
                .parse::<B256>()
                .map_err(|_| format!("block hash `{value}` is not a valid B256"))?;
            return Ok(BlockId::Hash(hash.into()));
        }
        value if value.starts_with("0x") => {
            let number = u64::from_str_radix(value.trim_start_matches("0x"), 16)
                .map_err(|error| format!("block hex parse error: {error}"))?;
            BlockNumberOrTag::Number(number)
        }
        value => {
            let number = value
                .parse::<u64>()
                .map_err(|error| format!("block tag `{value}` not recognized: {error}"))?;
            BlockNumberOrTag::Number(number)
        }
    };
    Ok(BlockId::Number(block))
}

/// Converts an Alloy transaction receipt into the core receipt contract.
pub(crate) fn alloy_to_cow_receipt(
    receipt: &alloy_rpc_types_eth::TransactionReceipt,
) -> Result<TransactionReceipt, AsyncProviderError> {
    let hash = format!("0x{:x}", receipt.transaction_hash);
    let hash = TransactionHash::new(hash)
        .map_err(|error| AsyncProviderError::Internal(format!("hash conversion: {error}")))?;
    Ok(TransactionReceipt::new(hash))
}

/// Converts an Alloy block response into the core block-info contract.
pub(crate) fn alloy_to_cow_block_info(
    block: &alloy_rpc_types_eth::Block,
) -> Result<BlockInfo, AsyncProviderError> {
    let number = block.header.number();
    let hash = cow_sdk_core::BlockHash::new(format!("0x{:x}", block.header.hash))
        .map_err(|error| AsyncProviderError::Internal(format!("hash conversion: {error}")))?;
    Ok(BlockInfo::new(number, Some(hash)))
}

pub(crate) fn parse_u256_quantity(value: &str, field: &str) -> Result<U256, AsyncProviderError> {
    value.strip_prefix("0x").map_or_else(
        || {
            U256::from_str_radix(value, 10).map_err(|error| {
                AsyncProviderError::Validation(format!(
                    "{field} `{value}` is not a valid U256: {error}"
                ))
            })
        },
        |hex| {
            U256::from_str_radix(hex, 16).map_err(|error| {
                AsyncProviderError::Validation(format!(
                    "{field} `{value}` is not a valid U256: {error}"
                ))
            })
        },
    )
}

pub(crate) fn hex_data_from_bytes(bytes: &[u8]) -> Result<HexData, AsyncProviderError> {
    HexData::new(format!("0x{}", hex::encode(bytes)))
        .map_err(|error| AsyncProviderError::Internal(format!("hex conversion: {error}")))
}

pub(crate) fn decode_0x_hex(value: &str) -> Result<Vec<u8>, String> {
    let stripped = value
        .strip_prefix("0x")
        .ok_or_else(|| "hex value must be 0x-prefixed".to_owned())?;
    hex::decode(stripped).map_err(|error| error.to_string())
}
