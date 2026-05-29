//! Event-log query and result types for the [`crate::LogProvider`] capability.
//!
//! These types describe a single [`crate::LogProvider::get_logs`] call: the
//! [`LogQuery`] filter the caller supplies, and the [`RawLog`] / [`LogMeta`]
//! results the adapter returns. They are transport-agnostic and carry no
//! provider or network dependency.

use alloy_primitives::LogData;

use super::identity::{Address, Hash32, TransactionHash};

/// Filter for a single on-chain event-log scan.
///
/// Describes the contract address, topic-0 (event-signature) candidates, and
/// the inclusive `[from_block, to_block]` range for one
/// [`crate::LogProvider::get_logs`] call. The block range is **caller-bounded**:
/// the SDK issues exactly one query over it and never expands it into an
/// open-ended or rolling scan.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogQuery {
    /// Contract address to scan; `None` matches logs from any address.
    pub address: Option<Address>,
    /// topic-0 (event-signature) candidates, matched as a set (any-of). An
    /// empty list matches any topic-0.
    pub topics: Vec<Hash32>,
    /// Inclusive first block of the scan range.
    pub from_block: u64,
    /// Inclusive last block of the scan range.
    pub to_block: u64,
}

impl LogQuery {
    /// Creates a query over the inclusive `[from_block, to_block]` range with no
    /// address or topic-0 filter.
    #[must_use]
    pub const fn new(from_block: u64, to_block: u64) -> Self {
        Self {
            address: None,
            topics: Vec::new(),
            from_block,
            to_block,
        }
    }

    /// Restricts the scan to a single contract address.
    #[must_use]
    pub const fn with_address(mut self, address: Address) -> Self {
        self.address = Some(address);
        self
    }

    /// Adds a topic-0 (event-signature) candidate to match.
    #[must_use]
    pub fn with_topic0(mut self, topic0: Hash32) -> Self {
        self.topics.push(topic0);
        self
    }
}

/// Positional metadata for a mined event log.
///
/// Populated for logs returned by a bounded historical scan. Pending logs are
/// out of scope: a [`LogProvider::get_logs`](crate::LogProvider::get_logs) call
/// over a numeric block range only returns mined logs.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogMeta {
    /// Number of the block that mined the log.
    pub block_number: u64,
    /// Hash of the transaction that emitted the log.
    pub transaction_hash: TransactionHash,
    /// Index of the log within its block.
    pub log_index: u64,
}

impl LogMeta {
    /// Creates log metadata from its parts.
    #[must_use]
    pub const fn new(block_number: u64, transaction_hash: TransactionHash, log_index: u64) -> Self {
        Self {
            block_number,
            transaction_hash,
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
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawLog {
    /// Address of the contract that emitted the log.
    pub address: Address,
    /// Indexed topics and non-indexed data of the log.
    pub data: LogData,
    /// Positional metadata for the log.
    pub meta: LogMeta,
}

impl RawLog {
    /// Creates a raw log from its parts.
    #[must_use]
    pub const fn new(address: Address, data: LogData, meta: LogMeta) -> Self {
        Self {
            address,
            data,
            meta,
        }
    }
}
