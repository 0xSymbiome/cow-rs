//! [`cow_sdk_core::AsyncProvider`] implementation over Alloy HTTP RPC.

use std::{fmt, sync::Arc};

use alloy_network::Ethereum;
use alloy_provider::{DynProvider, Provider as AlloyProviderTrait};
use cow_sdk_core::{
    Address, AsyncProvider, BlockInfo, ChainId, ContractCall, ContractHandle, HexData, Redacted,
    TransactionHash, TransactionReceipt, TransactionRequest,
};

use crate::{
    builder::RpcAlloyProviderBuilder,
    conversion::{
        alloy_to_cow_block_info, alloy_to_cow_receipt, cow_block_tag_to_alloy,
        cow_request_to_alloy, cow_to_alloy_address, cow_to_alloy_hash, hex_data_from_bytes,
        parse_u256_quantity,
    },
    error::AsyncProviderError,
    read_contract::execute_read_contract,
};

/// Alloy-backed read-only provider for the `CoW` Protocol Rust SDK.
///
/// The adapter implements only [`AsyncProvider`]. It does not implement signer
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

#[allow(async_fn_in_trait)]
impl AsyncProvider for RpcAlloyProvider {
    type Error = AsyncProviderError;

    async fn get_chain_id(&self) -> Result<ChainId, Self::Error> {
        self.inner()
            .get_chain_id()
            .await
            .map_err(AsyncProviderError::from_alloy_transport)
    }

    async fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error> {
        let address = cow_to_alloy_address(address)?;
        let bytes = self
            .inner()
            .get_code_at(address)
            .await
            .map_err(AsyncProviderError::from_alloy_transport)?;
        if bytes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(hex_data_from_bytes(bytes.as_ref())?))
        }
    }

    async fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        let hash = cow_to_alloy_hash(transaction_hash)?;
        self.inner()
            .get_transaction_receipt(hash)
            .await
            .map_err(AsyncProviderError::from_alloy_transport)?
            .map(|receipt| alloy_to_cow_receipt(&receipt))
            .transpose()
    }

    async fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error> {
        let address = cow_to_alloy_address(address)?;
        let slot = parse_u256_quantity(slot, "storage slot")?;
        let value = self
            .inner()
            .get_storage_at(address, slot)
            .await
            .map_err(AsyncProviderError::from_alloy_transport)?;
        HexData::new(format!("0x{value:064x}"))
            .map_err(|error| AsyncProviderError::Internal(format!("storage conversion: {error}")))
    }

    async fn call(&self, tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        let tx = cow_request_to_alloy(tx).map_err(AsyncProviderError::Validation)?;
        let bytes = self
            .inner()
            .call(tx)
            .await
            .map_err(AsyncProviderError::from_alloy_transport)?;
        hex_data_from_bytes(bytes.as_ref())
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        execute_read_contract(self.inner(), request).await
    }

    async fn get_block(&self, block_tag: &str) -> Result<BlockInfo, Self::Error> {
        let block_id = cow_block_tag_to_alloy(block_tag).map_err(AsyncProviderError::Validation)?;
        let block = self
            .inner()
            .get_block(block_id)
            .await
            .map_err(AsyncProviderError::from_alloy_transport)?
            .ok_or_else(|| {
                AsyncProviderError::Validation(format!("block `{block_tag}` not found on remote"))
            })?;
        alloy_to_cow_block_info(&block)
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(address.clone(), abi_json.to_owned()))
    }
}
