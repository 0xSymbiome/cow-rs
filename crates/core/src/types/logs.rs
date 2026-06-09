//! Event-log query and result types for the [`crate::LogProvider`] capability.
//!
//! These types describe a single [`crate::LogProvider::get_logs`] call: the
//! [`LogQuery`] filter the caller supplies, and the [`RawLog`] / [`LogMeta`]
//! results the adapter returns. They mirror the standard `eth_getLogs` shape —
//! an address set, four independent topic slots, and a block-number range or a
//! single block hash — while staying transport-agnostic with no provider or
//! network dependency.

use alloy_primitives::LogData;

use super::identity::{Address, BlockHash, Hash32, TransactionHash};

/// Block selector for a single bounded [`LogQuery`].
///
/// A scan targets either an inclusive block-number range or exactly one block
/// identified by hash. Both are single bounded queries: the SDK issues one
/// backend call and never expands or rolls the selection (ADR 0048). These are
/// the only two block selections `eth_getLogs` accepts, so the enum is
/// protocol-fixed and exhaustive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogBlockSelector {
    /// Inclusive `[from, to]` block-number range.
    Range {
        /// Inclusive first block of the scan range.
        from: u64,
        /// Inclusive last block of the scan range.
        to: u64,
    },
    /// Exactly the block with this hash (a reorg-stable single-block scan).
    Hash(BlockHash),
}

/// Filter for a single on-chain event-log scan.
///
/// Mirrors the standard `eth_getLogs` filter so a consumer can push every
/// predicate down to the node in one bounded call:
///
/// - [`addresses`](LogQuery::addresses): empty matches any contract; one or
///   many addresses match as an any-of set.
/// - [`topics`](LogQuery::topics): the four EVM topic slots. Slot 0 is the
///   event signature; slots 1-3 are the indexed event arguments in declaration
///   order. Each slot is matched as an any-of set, and an empty slot is a
///   wildcard (matches anything in that position).
/// - [`block`](LogQuery::block): an inclusive block-number range or a single
///   block hash.
///
/// The selection is **caller-bounded**: the SDK issues exactly one query over
/// it and never expands it into an open-ended or rolling scan (ADR 0048). A
/// caller that needs a wider range issues further bounded calls itself.
///
/// Every `CoW` on-chain event indexes its actor as the first indexed argument —
/// the `Trade` / `OrderInvalidated` / `PreSignature` owner, the `Settlement`
/// solver, the eth-flow sender / refunder — so the common "events for my
/// address" query is a topic-1 filter:
///
/// ```
/// use cow_sdk_core::{Address, Hash32, LogQuery};
///
/// # fn build(settlement: Address, trade_sig: Hash32, owner: Address) -> LogQuery {
/// LogQuery::new(20_000_000, 20_000_100)
///     .with_address(settlement)
///     .with_topic0(trade_sig)
///     .with_topic1(Hash32::from_indexed_address(&owner)) // only this owner's trades
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogQuery {
    /// Contract addresses to scan; empty matches logs from any address, and a
    /// non-empty list matches any of the addresses.
    pub addresses: Vec<Address>,
    /// The four EVM topic slots (topic-0 = event signature, topics 1-3 = the
    /// indexed arguments in order). Each slot matches as an any-of set; an
    /// empty slot is a wildcard.
    pub topics: [Vec<Hash32>; 4],
    /// Block-number range or single block hash to scan.
    pub block: LogBlockSelector,
}

impl LogQuery {
    /// Creates a query over the inclusive `[from_block, to_block]` block-number
    /// range with no address or topic filter.
    #[must_use]
    pub const fn new(from_block: u64, to_block: u64) -> Self {
        Self {
            addresses: Vec::new(),
            topics: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            block: LogBlockSelector::Range {
                from: from_block,
                to: to_block,
            },
        }
    }

    /// Creates a query over exactly the block identified by `block_hash`, with
    /// no address or topic filter.
    #[must_use]
    pub const fn at_block_hash(block_hash: BlockHash) -> Self {
        Self {
            addresses: Vec::new(),
            topics: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            block: LogBlockSelector::Hash(block_hash),
        }
    }

    /// Adds a contract address to the any-of address set.
    #[must_use]
    pub fn with_address(mut self, address: Address) -> Self {
        self.addresses.push(address);
        self
    }

    /// Adds every contract address from `addresses` to the any-of address set.
    #[must_use]
    pub fn with_addresses(mut self, addresses: impl IntoIterator<Item = Address>) -> Self {
        self.addresses.extend(addresses);
        self
    }

    /// Adds a topic-0 (event-signature) candidate to match.
    #[must_use]
    pub fn with_topic0(mut self, topic0: Hash32) -> Self {
        self.topics[0].push(topic0);
        self
    }

    /// Adds a candidate to the first indexed-argument slot (topic 1).
    ///
    /// For an indexed address argument (the common `CoW` case), build the topic
    /// with [`Hash32::from_indexed_address`].
    #[must_use]
    pub fn with_topic1(mut self, topic1: Hash32) -> Self {
        self.topics[1].push(topic1);
        self
    }

    /// Adds a candidate to the second indexed-argument slot (topic 2).
    #[must_use]
    pub fn with_topic2(mut self, topic2: Hash32) -> Self {
        self.topics[2].push(topic2);
        self
    }

    /// Adds a candidate to the third indexed-argument slot (topic 3).
    #[must_use]
    pub fn with_topic3(mut self, topic3: Hash32) -> Self {
        self.topics[3].push(topic3);
        self
    }
}

/// Positional metadata for a mined event log.
///
/// Populated for logs returned by a bounded historical scan; a
/// [`LogProvider::get_logs`](crate::LogProvider::get_logs) call over a
/// block-number range or a block hash returns only mined logs, so the block,
/// transaction, and index fields are always present. `block_timestamp` is
/// present only when the backend reports it.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogMeta {
    /// Number of the block that mined the log.
    pub block_number: u64,
    /// Hash of the block that mined the log.
    pub block_hash: BlockHash,
    /// Unix timestamp of the mining block, when the backend reports it.
    pub block_timestamp: Option<u64>,
    /// Hash of the transaction that emitted the log.
    pub transaction_hash: TransactionHash,
    /// Index of the emitting transaction within its block.
    pub transaction_index: u64,
    /// Index of the log within its block.
    pub log_index: u64,
}

impl LogMeta {
    /// Creates log metadata from its parts.
    #[must_use]
    pub const fn new(
        block_number: u64,
        block_hash: BlockHash,
        block_timestamp: Option<u64>,
        transaction_hash: TransactionHash,
        transaction_index: u64,
        log_index: u64,
    ) -> Self {
        Self {
            block_number,
            block_hash,
            block_timestamp,
            transaction_hash,
            transaction_index,
            log_index,
        }
    }
}

/// A single fetched event log, ready to decode.
///
/// `data` carries the indexed topics and non-indexed bytes in the shape the
/// fail-closed `cow-sdk-contracts` decoders consume (`decode_settlement_log`,
/// `decode_eth_flow_log`, and the other `decode_*_log` functions); pass
/// `&raw_log.data` straight to one of them.
///
/// `removed` reports whether the backend marked this log as removed by a chain
/// reorganization, so a consumer composing its own watcher from successive
/// bounded calls can reconcile reorged logs.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawLog {
    /// Address of the contract that emitted the log.
    pub address: Address,
    /// Indexed topics and non-indexed data of the log.
    pub data: LogData,
    /// Positional metadata for the log.
    pub meta: LogMeta,
    /// Whether the backend marked this log removed by a chain reorganization.
    pub removed: bool,
}

impl RawLog {
    /// Creates a raw log from its parts.
    #[must_use]
    pub const fn new(address: Address, data: LogData, meta: LogMeta, removed: bool) -> Self {
        Self {
            address,
            data,
            meta,
            removed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_builds_a_bounded_range_with_no_filters() {
        let query = LogQuery::new(100, 200);
        assert!(query.addresses.is_empty());
        assert!(query.topics.iter().all(Vec::is_empty));
        assert_eq!(query.block, LogBlockSelector::Range { from: 100, to: 200 });
    }

    #[test]
    fn at_block_hash_selects_a_single_block() {
        let block_hash = Hash32::from_bytes([0x33; 32]);
        let query = LogQuery::at_block_hash(block_hash);
        assert_eq!(query.block, LogBlockSelector::Hash(block_hash));
    }

    #[test]
    fn builders_populate_addresses_and_topic_slots() {
        let first = Address::new("0x1111111111111111111111111111111111111111").unwrap();
        let second = Address::new("0x2222222222222222222222222222222222222222").unwrap();
        let owner = Address::new("0x3333333333333333333333333333333333333333").unwrap();

        let query = LogQuery::new(1, 2)
            .with_address(first)
            .with_addresses([second])
            .with_topic0(Hash32::from_bytes([0xaa; 32]))
            .with_topic1(Hash32::from_indexed_address(&owner));

        assert_eq!(query.addresses.len(), 2);
        assert_eq!(query.topics[0].len(), 1);
        assert_eq!(query.topics[1].len(), 1);
        assert!(query.topics[2].is_empty());
        assert!(query.topics[3].is_empty());
    }

    #[test]
    fn from_indexed_address_left_pads_to_a_topic() {
        let owner = Address::new("0x000000000000000000000000000000000000dead").unwrap();
        let topic = Hash32::from_indexed_address(&owner);
        let bytes = topic.as_slice();
        // High 12 bytes are zero; the 20 address bytes are right-aligned.
        assert!(bytes[..12].iter().all(|byte| *byte == 0));
        assert_eq!(&bytes[12..], owner.as_alloy().as_slice());
    }
}
