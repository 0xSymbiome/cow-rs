//! Owned signer handle returned by [`crate::AlloyClient`].

use std::{fmt, sync::Arc};

use alloy_provider::Provider as AlloyProviderTrait;
use alloy_signer::Signer as AlloySigner;
use cow_sdk_core::{
    Address, Amount, AsyncSigner, TransactionBroadcast, TransactionHash, TransactionRequest,
    TypedDataDomain, TypedDataField, TypedDataPayload,
};

use crate::{
    client::AlloyClientInner,
    conversion::{
        alloy_signature_to_hex, cow_flat_to_alloy_typed_data, cow_request_to_alloy,
        cow_typed_data_payload_to_alloy,
    },
    error::AlloyClientError,
};

/// Owned async signer handle created from [`crate::AlloyClient`].
///
/// The handle clones the client's internal `Arc`, so it remains usable even if
/// the parent client value is dropped.
#[derive(Clone)]
pub struct AlloyClientSignerHandle {
    inner: Arc<AlloyClientInner>,
}

impl AlloyClientSignerHandle {
    pub(crate) const fn new(inner: Arc<AlloyClientInner>) -> Self {
        Self { inner }
    }

    /// Returns the chain id bound to the underlying local signer.
    #[must_use]
    pub fn chain_id(&self) -> cow_sdk_core::ChainId {
        self.inner.chain_id
    }
}

impl fmt::Debug for AlloyClientSignerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AlloyClientSignerHandle")
            .field("chain_id", &self.inner.chain_id)
            .field("signer_address", &self.inner.signer_address)
            .field("signer", &"[redacted]")
            .finish()
    }
}

impl AsyncSigner for AlloyClientSignerHandle {
    type Error = AlloyClientError;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.inner.signer_address.clone())
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        let signature = AlloySigner::sign_message(&self.inner.signer, message)
            .await
            .map_err(|error| AlloyClientError::from_alloy_signer(&error))?;
        Ok(alloy_signature_to_hex(&signature)?)
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Err(AlloyClientError::UnsupportedTransactionRequest {
            method: "sign_transaction",
            reason: "raw transaction signing is deferred; use send_transaction for on-chain operations",
        })
    }

    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        #[cfg(feature = "eip712")]
        {
            let typed =
                cow_typed_data_payload_to_alloy(payload).map_err(AlloyClientError::Validation)?;
            let signature = AlloySigner::sign_dynamic_typed_data(&self.inner.signer, &typed)
                .await
                .map_err(|error| AlloyClientError::from_alloy_signer(&error))?;
            Ok(alloy_signature_to_hex(&signature)?)
        }

        #[cfg(not(feature = "eip712"))]
        {
            let _ = payload;
            Err(AlloyClientError::Validation(
                "sign_typed_data_payload requires the eip712 feature".to_owned(),
            ))
        }
    }

    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error> {
        #[cfg(feature = "eip712")]
        {
            let typed = cow_flat_to_alloy_typed_data(domain, fields, value_json)
                .map_err(AlloyClientError::Validation)?;
            let signature = AlloySigner::sign_dynamic_typed_data(&self.inner.signer, &typed)
                .await
                .map_err(|error| AlloyClientError::from_alloy_signer(&error))?;
            Ok(alloy_signature_to_hex(&signature)?)
        }

        #[cfg(not(feature = "eip712"))]
        {
            let _ = (domain, fields, value_json);
            Err(AlloyClientError::Validation(
                "sign_typed_data requires the eip712 feature".to_owned(),
            ))
        }
    }

    /// Submits a transaction through the wallet-filler provider and
    /// returns the broadcast acknowledgement.
    ///
    /// The returned [`TransactionBroadcast`] confirms the broadcast was
    /// accepted by the underlying Alloy provider; it does not prove inclusion
    /// or execution success. Use
    /// [`cow_sdk_core::AsyncProvider::get_transaction_receipt`] or a
    /// higher-level `cow-sdk-trading` wait helper to observe mined status.
    /// The umbrella reads the broadcast hash through
    /// `pending.tx_hash()`, Alloy's immediate accessor, so it does not wait
    /// for confirmation before returning.
    async fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        let tx = cow_request_to_alloy(tx).map_err(AlloyClientError::Validation)?;
        let pending = self
            .inner
            .provider
            .send_transaction(tx)
            .await
            .map_err(AlloyClientError::from_alloy_transport)?;
        let tx_hash = *pending.tx_hash();
        let transaction_hash = TransactionHash::new(format!("0x{tx_hash:x}"))
            .map_err(|error| AlloyClientError::Internal(format!("hash conversion: {error}")))?;
        Ok(TransactionBroadcast::new(transaction_hash))
    }

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        let tx = cow_request_to_alloy(tx).map_err(AlloyClientError::Validation)?;
        let gas = self
            .inner
            .provider
            .estimate_gas(tx)
            .await
            .map_err(AlloyClientError::from_alloy_transport)?;
        Ok(Amount::from(gas))
    }
}
