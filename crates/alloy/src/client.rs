//! Composed Alloy client implementing provider and signing-provider traits.

use std::{fmt, sync::Arc};

use alloy_network::Ethereum;
use alloy_provider::{DynProvider, Provider as _, ProviderBuilder};
use alloy_signer::Signer as AlloySigner;
use alloy_signer_local::PrivateKeySigner;
use cow_sdk_core::{
    Address, AsyncProvider, AsyncSigningProvider, BlockInfo, ChainId, ContractCall, ContractHandle,
    HexData, TransactionHash, TransactionReceipt, TransactionRequest,
};

use alloy_primitives::{B256, U256};
use cow_sdk_alloy_provider::__seam::execute_read_contract as execute_read_contract_seam;

use crate::{
    builder::AlloyClientBuilder,
    conversion::{
        alloy_to_cow_block_info, alloy_to_cow_receipt, cow_block_tag_to_alloy, cow_request_to_alloy,
    },
    error::AlloyClientError,
    handle::AlloyClientSignerHandle,
};

pub(crate) struct AlloyClientInner {
    pub(crate) provider: DynProvider<Ethereum>,
    pub(crate) signer: PrivateKeySigner,
    pub(crate) signer_address: Address,
    pub(crate) chain_id: ChainId,
}

/// Native composed Alloy provider and signer client.
///
/// `AlloyClient` owns a wallet-filler Alloy provider and a concrete local
/// signer. It implements [`AsyncProvider`] for read-only RPC calls and
/// [`AsyncSigningProvider`] for creating an owned [`AlloyClientSignerHandle`].
#[derive(Clone)]
pub struct AlloyClient {
    pub(crate) inner: Arc<AlloyClientInner>,
}

impl AlloyClient {
    /// Returns a fresh typestate builder.
    pub const fn builder() -> AlloyClientBuilder {
        AlloyClientBuilder::new()
    }

    pub(crate) fn from_parts(
        rpc_url: reqwest::Url,
        signer: PrivateKeySigner,
        chain_id: ChainId,
    ) -> Self {
        let signer = signer.with_chain_id(Some(chain_id));
        let alloy_address = AlloySigner::address(&signer);
        let signer_address = Address::from_bytes(alloy_address.into_array());
        let provider = ProviderBuilder::new()
            .wallet(signer.clone())
            .connect_http(rpc_url)
            .erased();

        Self {
            inner: Arc::new(AlloyClientInner {
                provider,
                signer,
                signer_address,
                chain_id,
            }),
        }
    }

    /// Returns the signer address cached at construction time.
    #[must_use]
    pub fn signer_address(&self) -> Address {
        self.inner.signer_address
    }

    /// Returns the chain id bound to the signer at construction time.
    #[must_use]
    pub fn chain_id(&self) -> ChainId {
        self.inner.chain_id
    }

    /// Verifies that the configured chain id matches the chain id reported by
    /// the RPC endpoint.
    ///
    /// # Errors
    ///
    /// Returns the underlying transport error for RPC failure. Returns
    /// [`AlloyClientError::Validation`] when the configured chain id does not
    /// match the remote endpoint.
    pub async fn verify_chain_id(&self) -> Result<(), AlloyClientError> {
        let remote = self.get_chain_id().await?;
        if remote != self.inner.chain_id {
            return Err(AlloyClientError::Validation(format!(
                "configured chain id `{}` does not match remote `eth_chainId` `{remote}`",
                self.inner.chain_id
            )));
        }
        Ok(())
    }
}

impl fmt::Debug for AlloyClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AlloyClient")
            .field("chain_id", &self.inner.chain_id)
            .field("signer_address", &self.inner.signer_address)
            .field("transport", &"<redacted>")
            .finish()
    }
}

impl AsyncProvider for AlloyClient {
    type Error = AlloyClientError;

    async fn get_chain_id(&self) -> Result<ChainId, Self::Error> {
        self.inner
            .provider
            .get_chain_id()
            .await
            .map_err(AlloyClientError::from_alloy_transport)
    }

    async fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error> {
        let bytes = self
            .inner
            .provider
            .get_code_at(*address.as_alloy())
            .await
            .map_err(AlloyClientError::from_alloy_transport)?;
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
            .inner
            .provider
            .get_transaction_receipt(*transaction_hash.as_alloy())
            .await
            .map_err(AlloyClientError::from_alloy_transport)?
            .map(|receipt| alloy_to_cow_receipt(&receipt));
        Ok(receipt)
    }

    async fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error> {
        let slot = slot
            .strip_prefix("0x")
            .map_or_else(
                || U256::from_str_radix(slot, 10),
                |hex| U256::from_str_radix(hex, 16),
            )
            .map_err(|error| {
                AlloyClientError::Validation(format!(
                    "storage slot `{slot}` is not a valid U256: {error}"
                ))
            })?;
        let value = self
            .inner
            .provider
            .get_storage_at(*address.as_alloy(), slot)
            .await
            .map_err(AlloyClientError::from_alloy_transport)?;
        HexData::new(B256::from(value).to_string())
            .map_err(|error| AlloyClientError::Internal(format!("storage conversion: {error}")))
    }

    async fn call(&self, tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        let tx = cow_request_to_alloy(tx).map_err(AlloyClientError::Validation)?;
        let bytes = self
            .inner
            .provider
            .call(tx)
            .await
            .map_err(AlloyClientError::from_alloy_transport)?;
        Ok(HexData::from(bytes))
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        execute_read_contract_seam(&self.inner.provider, request)
            .await
            .map_err(AlloyClientError::from)
    }

    async fn get_block(&self, block_tag: &str) -> Result<BlockInfo, Self::Error> {
        let block_id = cow_block_tag_to_alloy(block_tag).map_err(AlloyClientError::Validation)?;
        let block = self
            .inner
            .provider
            .get_block(block_id)
            .await
            .map_err(AlloyClientError::from_alloy_transport)?
            .ok_or_else(|| {
                AlloyClientError::Validation(format!("block `{block_tag}` not found on remote"))
            })?;
        Ok(alloy_to_cow_block_info(&block))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(*address, abi_json.to_owned()))
    }
}

impl AsyncSigningProvider for AlloyClient {
    type Signer = AlloyClientSignerHandle;

    async fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(AlloyClientSignerHandle::new(Arc::clone(&self.inner)))
    }
}
