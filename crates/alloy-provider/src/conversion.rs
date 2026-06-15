//! Conversion helpers between `cow-sdk-core` values and Alloy RPC values.

use alloy_consensus::{BlockHeader as _, TxReceipt as _};
use alloy_network::TransactionBuilder;
use alloy_primitives::{Address as AlloyAddress, B256, U256};
use alloy_rpc_types_eth::{
    BlockId, BlockNumberOrTag, Filter, Log as AlloyLog, TransactionRequest as AlloyTransaction,
};
use cow_sdk_core::{
    Address, Amount, BlockHash, BlockInfo, Hash32, LogBlockSelector, LogMeta, LogQuery, RawLog,
    TransactionHash, TransactionReceipt, TransactionRequest, TransactionStatus,
};

/// Converts a core transaction request into an Alloy transaction request.
pub(crate) fn cow_request_to_alloy(
    request: &TransactionRequest,
) -> Result<AlloyTransaction, String> {
    let mut alloy_tx = AlloyTransaction::default();
    if let Some(to) = &request.to {
        alloy_tx = alloy_tx.with_to(*to.as_alloy());
    }
    if let Some(data) = &request.data {
        alloy_tx = alloy_tx.with_input(data.as_alloy().clone());
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
                .parse::<alloy_primitives::B256>()
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

/// Converts an Alloy transaction receipt into the SDK receipt contract.
///
/// Populates every rich field the Alloy receipt carries. Status is read
/// through the consensus receipt's EIP-658 accessor so post-state receipts
/// surface as `None` rather than coerced success. The `to` field is `None` for
/// contract-creation transactions.
pub(crate) fn alloy_to_cow_receipt(
    receipt: &alloy_rpc_types_eth::TransactionReceipt,
) -> TransactionReceipt {
    let transaction_hash = TransactionHash::from(receipt.transaction_hash);

    let status = receipt
        .inner
        .status_or_post_state()
        .as_eip658()
        .map(|success| {
            if success {
                TransactionStatus::Success
            } else {
                TransactionStatus::Reverted
            }
        });

    let block_hash = receipt.block_hash.map(BlockHash::from);
    let from = Some(Address::from(receipt.from));
    let to = receipt.to.map(Address::from);

    TransactionReceipt::from_parts(
        transaction_hash,
        status,
        receipt.block_number,
        block_hash,
        Some(Amount::from(receipt.gas_used)),
        from,
        to,
    )
}

/// Converts an Alloy block response into the core block-info contract.
pub(crate) fn alloy_to_cow_block_info(block: &alloy_rpc_types_eth::Block) -> BlockInfo {
    let number = block.header.number();
    let hash = BlockHash::from(block.header.hash);
    BlockInfo::new(number, Some(hash))
}

/// Converts a core [`LogQuery`] into an Alloy `eth_getLogs` filter.
///
/// Maps the block selector (number range or block hash), the optional address
/// set, and all four topic slots. An empty address set or topic slot leaves
/// that dimension unconstrained (match-any).
pub(crate) fn cow_log_query_to_alloy_filter(query: &LogQuery) -> Filter {
    let mut filter = match &query.block {
        LogBlockSelector::Range { from, to } => Filter::new().from_block(*from).to_block(*to),
        LogBlockSelector::Hash(hash) => Filter::new().at_block_hash(*hash.as_alloy()),
    };
    if !query.addresses.is_empty() {
        let addresses: Vec<AlloyAddress> = query
            .addresses
            .iter()
            .map(|address| *address.as_alloy())
            .collect();
        filter = filter.address(addresses);
    }
    if !query.topics[0].is_empty() {
        filter = filter.event_signature(topic_filter_set(&query.topics[0]));
    }
    if !query.topics[1].is_empty() {
        filter = filter.topic1(topic_filter_set(&query.topics[1]));
    }
    if !query.topics[2].is_empty() {
        filter = filter.topic2(topic_filter_set(&query.topics[2]));
    }
    if !query.topics[3].is_empty() {
        filter = filter.topic3(topic_filter_set(&query.topics[3]));
    }
    filter
}

/// Maps a core topic slot (an any-of set of [`Hash32`]) into the Alloy `B256`
/// values its `Topic` filter accepts.
fn topic_filter_set(values: &[Hash32]) -> Vec<B256> {
    values.iter().map(|value| *value.as_alloy()).collect()
}

/// Converts an Alloy log into the core [`RawLog`] contract.
///
/// Carries the full mined-log metadata — block number and hash, optional block
/// timestamp, transaction hash and index, and log index — plus the reorg
/// `removed` flag. A bounded historical scan only returns mined logs, so the
/// always-present fields fall back to zero only on a malformed response.
pub(crate) fn alloy_log_to_cow_raw_log(log: &AlloyLog) -> RawLog {
    let meta = LogMeta::new(
        log.block_number.unwrap_or_default(),
        log.block_hash.map_or(BlockHash::ZERO, BlockHash::from),
        log.block_timestamp,
        log.transaction_hash
            .map_or(TransactionHash::ZERO, TransactionHash::from),
        log.transaction_index.unwrap_or_default(),
        log.log_index.unwrap_or_default(),
    );
    RawLog::new(
        Address::from(log.inner.address),
        log.inner.data.clone(),
        meta,
        log.removed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_rpc_types_eth::TransactionReceipt as AlloyTransactionReceipt;
    use serde_json::{Value, json};

    const HASH_1: &str = "0x1111111111111111111111111111111111111111111111111111111111111111";
    const BLOCK_HASH: &str = "0x3333333333333333333333333333333333333333333333333333333333333333";
    const ROOT_HASH: &str = "0x4444444444444444444444444444444444444444444444444444444444444444";
    const FROM_ADDR: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
    const TO_ADDR: &str = "0x70997970c51812dc3a010c7d01b50e0d17dc79c8";

    #[test]
    fn alloy_to_cow_receipt_populates_status_success() {
        let receipt = alloy_receipt(&json!({ "status": "0x1" }));

        let cow_receipt = alloy_to_cow_receipt(&receipt);

        assert_eq!(cow_receipt.transaction_hash.to_hex_string(), HASH_1);
        assert_eq!(cow_receipt.status, Some(TransactionStatus::Success));
        assert_eq!(cow_receipt.block_number, Some(1234));
        assert_eq!(cow_receipt.block_hash.unwrap().to_hex_string(), BLOCK_HASH);
        assert_eq!(cow_receipt.gas_used, Some(Amount::from(21_000u64)));
        assert_eq!(cow_receipt.from.unwrap().to_hex_string(), FROM_ADDR);
        assert_eq!(cow_receipt.to.unwrap().to_hex_string(), TO_ADDR);
    }

    #[test]
    fn alloy_to_cow_receipt_populates_status_reverted() {
        let receipt = alloy_receipt(&json!({ "status": "0x0" }));

        let cow_receipt = alloy_to_cow_receipt(&receipt);

        assert_eq!(cow_receipt.status, Some(TransactionStatus::Reverted));
    }

    #[test]
    fn alloy_to_cow_receipt_returns_none_status_for_post_state_receipt() {
        let receipt = alloy_receipt(&json!({ "root": ROOT_HASH }));
        assert_eq!(receipt.inner.status_or_post_state().as_eip658(), None);

        let cow_receipt = alloy_to_cow_receipt(&receipt);

        assert_eq!(cow_receipt.status, None);
    }

    #[test]
    fn alloy_to_cow_receipt_handles_contract_creation_no_to() {
        let receipt = alloy_receipt_with_to(&json!({ "status": "0x1" }), &Value::Null);

        let cow_receipt = alloy_to_cow_receipt(&receipt);

        assert!(cow_receipt.to.is_none());
        assert_eq!(cow_receipt.from.unwrap().to_hex_string(), FROM_ADDR);
    }

    #[test]
    fn cow_log_query_to_alloy_filter_sets_caller_bounded_range() {
        let query = LogQuery::new(100, 200);
        let filter = cow_log_query_to_alloy_filter(&query);
        assert_eq!(filter.get_from_block(), Some(100));
        assert_eq!(filter.get_to_block(), Some(200));
    }

    #[test]
    fn cow_log_query_to_alloy_filter_maps_topics_addresses_and_block_hash() {
        // Range query: address list + topic0 + an indexed-owner topic1.
        let owner = Address::new("0x000000000000000000000000000000000000dead").unwrap();
        let first = Address::new("0x1111111111111111111111111111111111111111").unwrap();
        let second = Address::new("0x2222222222222222222222222222222222222222").unwrap();
        let query = LogQuery::new(10, 20)
            .with_addresses([first, second])
            .with_topic0(Hash32::from_bytes([0xaa; 32]))
            .with_topic1(Hash32::from_indexed_address(&owner));
        let filter = cow_log_query_to_alloy_filter(&query);

        assert_eq!(filter.address.len(), 2);
        assert!(!filter.topics[0].is_empty());
        assert!(!filter.topics[1].is_empty(), "indexed owner maps to topic1");
        assert!(filter.topics[2].is_empty());
        assert!(filter.topics[3].is_empty());
        assert_eq!(filter.get_from_block(), Some(10));
        assert_eq!(filter.get_to_block(), Some(20));

        // Block-hash query maps to at_block_hash.
        let by_hash =
            cow_log_query_to_alloy_filter(&LogQuery::at_block_hash(Hash32::from_bytes([0x33; 32])));
        assert!(by_hash.get_block_hash().is_some());
    }

    #[test]
    fn alloy_log_to_cow_raw_log_maps_address_meta_and_payload() {
        let log: AlloyLog = serde_json::from_value(json!({
            "address": FROM_ADDR,
            "topics": [HASH_1],
            "data": "0x",
            "blockHash": BLOCK_HASH,
            "blockNumber": "0x4d2",
            "transactionHash": HASH_1,
            "transactionIndex": "0x0",
            "logIndex": "0x2",
            "removed": false
        }))
        .expect("log fixture must deserialize");

        let raw = alloy_log_to_cow_raw_log(&log);

        assert_eq!(raw.address.to_hex_string(), FROM_ADDR);
        assert_eq!(raw.meta.block_number, 1234);
        assert_eq!(raw.meta.block_hash.to_hex_string(), BLOCK_HASH);
        assert_eq!(raw.meta.transaction_hash.to_hex_string(), HASH_1);
        assert_eq!(raw.meta.transaction_index, 0);
        assert_eq!(raw.meta.log_index, 2);
        assert!(!raw.removed);
        assert_eq!(raw.data.topics().len(), 1);
    }

    fn alloy_receipt(status_or_root: &Value) -> AlloyTransactionReceipt {
        alloy_receipt_with_to(status_or_root, &json!(TO_ADDR))
    }

    fn alloy_receipt_with_to(status_or_root: &Value, to: &Value) -> AlloyTransactionReceipt {
        let mut receipt = json!({
            "transactionHash": HASH_1,
            "transactionIndex": "0x0",
            "blockHash": BLOCK_HASH,
            "blockNumber": "0x4d2",
            "from": FROM_ADDR,
            "to": to,
            "contractAddress": null,
            "gasUsed": "0x5208",
            "effectiveGasPrice": "0x1",
            "cumulativeGasUsed": "0x5208",
            "logsBloom": format!("0x{}", "00".repeat(256)),
            "logs": [],
            "type": "0x2"
        });
        let object = receipt
            .as_object_mut()
            .expect("receipt fixture must be an object");
        for (key, value) in status_or_root
            .as_object()
            .expect("status fixture must be an object")
        {
            object.insert(key.clone(), value.clone());
        }
        serde_json::from_value(receipt).unwrap()
    }
}
