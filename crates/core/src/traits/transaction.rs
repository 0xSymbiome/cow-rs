use serde::{Deserialize, Serialize};

use crate::types::{Address, Amount, BlockHash, HexData, TransactionHash};
/// Transaction request shape used across signer and provider traits.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Destination address for the transaction.
    pub to: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Hex-encoded calldata payload.
    pub data: Option<HexData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Native token value to transfer.
    pub value: Option<Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional gas limit override.
    pub gas_limit: Option<Amount>,
}

impl TransactionRequest {
    /// Creates a transaction request shape.
    #[inline]
    #[must_use]
    pub const fn new(
        to: Option<Address>,
        data: Option<HexData>,
        value: Option<Amount>,
        gas_limit: Option<Amount>,
    ) -> Self {
        Self {
            to,
            data,
            value,
            gas_limit,
        }
    }
}

/// Broadcast acknowledgement returned by signer-backed transaction submission.
///
/// This value confirms that a backend accepted or observed a transaction hash.
/// It does not imply that the transaction has been mined, succeeded, or even
/// become visible to a read provider. Use [`crate::Provider::get_transaction_receipt`],
/// [`crate::AsyncProvider::get_transaction_receipt`], or a higher-level
/// `cow-sdk-trading` wait helper when lifecycle state is required.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionBroadcast {
    /// Transaction hash for the submitted transaction.
    pub transaction_hash: TransactionHash,
}

impl TransactionBroadcast {
    /// Creates a transaction broadcast acknowledgement from its hash.
    #[inline]
    #[must_use]
    pub const fn new(transaction_hash: TransactionHash) -> Self {
        Self { transaction_hash }
    }
}

/// Terminal transaction execution state exposed by receipt-capable providers.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionStatus {
    /// The transaction was mined successfully.
    Success,
    /// The transaction was mined and reverted.
    Reverted,
}

/// Transaction receipt contract returned by provider receipt lookups.
///
/// [`TransactionReceipt::new`] preserves hash-only adapters by leaving every
/// rich lifecycle field empty. Receipt-capable providers can populate the
/// optional fields with [`TransactionReceipt::from_parts`] or the builder
/// methods as adapter support matures.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    /// Transaction hash for the observed transaction.
    pub transaction_hash: TransactionHash,
    /// Optional terminal execution status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TransactionStatus>,
    /// Optional block number that included the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,
    /// Optional block hash that included the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<BlockHash>,
    /// Optional gas used by the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_used: Option<Amount>,
    /// Optional sender address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// Optional destination address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,
}

impl TransactionReceipt {
    /// Creates a hash-only transaction receipt.
    #[inline]
    #[must_use]
    pub const fn new(transaction_hash: TransactionHash) -> Self {
        Self {
            transaction_hash,
            status: None,
            block_number: None,
            block_hash: None,
            gas_used: None,
            from: None,
            to: None,
        }
    }

    /// Creates a transaction receipt from every supported receipt field.
    #[inline]
    #[must_use]
    pub const fn from_parts(
        transaction_hash: TransactionHash,
        status: Option<TransactionStatus>,
        block_number: Option<u64>,
        block_hash: Option<BlockHash>,
        gas_used: Option<Amount>,
        from: Option<Address>,
        to: Option<Address>,
    ) -> Self {
        Self {
            transaction_hash,
            status,
            block_number,
            block_hash,
            gas_used,
            from,
            to,
        }
    }

    /// Sets the terminal execution status.
    #[must_use]
    pub const fn with_status(mut self, status: TransactionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the block number that included the transaction.
    #[must_use]
    pub const fn with_block_number(mut self, block_number: u64) -> Self {
        self.block_number = Some(block_number);
        self
    }

    /// Sets the block hash that included the transaction.
    #[must_use]
    pub fn with_block_hash(mut self, block_hash: BlockHash) -> Self {
        self.block_hash = Some(block_hash);
        self
    }

    /// Sets the gas used by the transaction.
    #[must_use]
    pub fn with_gas_used(mut self, gas_used: Amount) -> Self {
        self.gas_used = Some(gas_used);
        self
    }

    /// Sets the sender address.
    #[must_use]
    pub fn with_from(mut self, from: Address) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the destination address.
    #[must_use]
    pub fn with_to(mut self, to: Address) -> Self {
        self.to = Some(to);
        self
    }
}

/// Minimal block information contract used by provider traits.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockInfo {
    /// Block number.
    pub number: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional block hash when the backend returns it.
    pub hash: Option<BlockHash>,
}

impl BlockInfo {
    /// Creates minimal block information.
    #[inline]
    #[must_use]
    pub const fn new(number: u64, hash: Option<BlockHash>) -> Self {
        Self { number, hash }
    }
}
