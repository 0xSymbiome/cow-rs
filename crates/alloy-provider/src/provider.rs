//! [`cow_sdk_core::Provider`] implementation over Alloy HTTP RPC.

use std::{fmt, sync::Arc};

use alloy_network::Ethereum;
use alloy_provider::{DynProvider, Provider as AlloyProviderTrait};
use cow_sdk_core::{
    Address, BlockInfo, ChainId, ContractCall, HexData, LogProvider, LogQuery, Provider, RawLog,
    Redacted, TransactionHash, TransactionReceipt, TransactionRequest,
};

use crate::{
    builder::RpcAlloyProviderBuilder,
    conversion::{
        alloy_log_to_cow_raw_log, alloy_to_cow_block_info, alloy_to_cow_receipt,
        cow_block_tag_to_alloy, cow_log_query_to_alloy_filter, cow_request_to_alloy,
    },
    error::ProviderError,
    read_contract::execute_read_contract,
};

/// Alloy-backed read-only provider for the `CoW` Protocol Rust SDK.
///
/// The adapter implements only [`Provider`]. It does not implement signer
/// creation, synchronous provider traits, or signer traits.
#[derive(Clone)]
pub struct RpcAlloyProvider {
    inner: Arc<DynProvider<Ethereum>>,
    transport: Redacted<String>,
}

impl RpcAlloyProvider {
    /// Returns a new typestate builder.
    pub const fn builder() -> RpcAlloyProviderBuilder {
        RpcAlloyProviderBuilder::new()
    }

    pub(crate) const fn from_parts(
        inner: Arc<DynProvider<Ethereum>>,
        transport: Redacted<String>,
    ) -> Self {
        Self { inner, transport }
    }

    pub(crate) fn inner(&self) -> &DynProvider<Ethereum> {
        &self.inner
    }

    /// Returns the redacted transport label used by debug output.
    #[must_use]
    pub const fn transport_label(&self) -> &Redacted<String> {
        &self.transport
    }
}

impl fmt::Debug for RpcAlloyProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RpcAlloyProvider")
            .field("chain_id", &"<lazy>")
            .field("transport", &OpaqueTransport)
            .finish()
    }
}

struct OpaqueTransport;

impl fmt::Debug for OpaqueTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl Provider for RpcAlloyProvider {
    type Error = ProviderError;

    async fn get_chain_id(&self) -> Result<ChainId, Self::Error> {
        self.inner()
            .get_chain_id()
            .await
            .map_err(ProviderError::from_alloy_transport)
    }

    async fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error> {
        let bytes = self
            .inner()
            .get_code_at(*address.as_alloy())
            .await
            .map_err(ProviderError::from_alloy_transport)?;
        if bytes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(HexData::from(bytes)))
        }
    }

    async fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        let receipt = self
            .inner()
            .get_transaction_receipt(*transaction_hash.as_alloy())
            .await
            .map_err(ProviderError::from_alloy_transport)?
            .map(|receipt| alloy_to_cow_receipt(&receipt));
        Ok(receipt)
    }

    async fn call(&self, tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        let tx = cow_request_to_alloy(tx).map_err(ProviderError::Validation)?;
        let bytes = self
            .inner()
            .call(tx)
            .await
            .map_err(ProviderError::from_alloy_transport)?;
        Ok(HexData::from(bytes))
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        execute_read_contract(self.inner(), request).await
    }

    async fn get_block(&self, block_tag: &str) -> Result<BlockInfo, Self::Error> {
        let block_id = cow_block_tag_to_alloy(block_tag).map_err(ProviderError::Validation)?;
        let block = self
            .inner()
            .get_block(block_id)
            .await
            .map_err(ProviderError::from_alloy_transport)?
            .ok_or_else(|| {
                ProviderError::Validation(format!("block `{block_tag}` not found on remote"))
            })?;
        Ok(alloy_to_cow_block_info(&block))
    }
}

impl LogProvider for RpcAlloyProvider {
    async fn get_logs(&self, query: &LogQuery) -> Result<Vec<RawLog>, Self::Error> {
        let filter = cow_log_query_to_alloy_filter(query);
        let logs = self
            .inner()
            .get_logs(&filter)
            .await
            .map_err(ProviderError::from_alloy_transport)?;
        Ok(logs.iter().map(alloy_log_to_cow_raw_log).collect())
    }
}
