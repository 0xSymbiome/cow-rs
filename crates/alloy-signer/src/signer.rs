//! `Signer` implementation backed by Alloy's local private-key signer.

use std::{fmt, sync::Arc};

use alloy_signer::Signer as AlloySigner;
use alloy_signer_local::PrivateKeySigner;
use cow_sdk_core::{
    Address, Amount, Signer, ChainId, TransactionBroadcast, TransactionRequest,
    TypedDataDomain, TypedDataField, TypedDataPayload,
};

use crate::{
    builder::LocalAlloyKeystoreSignerBuilder,
    conversion::{
        alloy_signature_to_hex, cow_flat_to_alloy_typed_data, cow_typed_data_payload_to_alloy,
    },
    error::SignerError,
};

/// Alloy-backed local-keystore signer for native SDK consumers.
#[derive(Clone)]
pub struct LocalAlloyKeystoreSigner {
    inner: Arc<PrivateKeySigner>,
    chain_id: ChainId,
}

impl LocalAlloyKeystoreSigner {
    /// Returns a fresh typestate builder.
    pub const fn builder() -> LocalAlloyKeystoreSignerBuilder {
        LocalAlloyKeystoreSignerBuilder::new()
    }

    pub(crate) fn from_parts(inner: PrivateKeySigner, chain_id: ChainId) -> Self {
        let bound = inner.with_chain_id(Some(chain_id));
        Self {
            inner: Arc::new(bound),
            chain_id,
        }
    }

    /// Returns the chain id bound to this signer at construction time.
    #[must_use]
    pub const fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub(crate) fn upstream_signer(&self) -> &PrivateKeySigner {
        &self.inner
    }
}

impl fmt::Debug for LocalAlloyKeystoreSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalAlloyKeystoreSigner")
            .field("inner", &"[redacted]")
            .field("chain_id", &self.chain_id)
            .finish()
    }
}

impl Signer for LocalAlloyKeystoreSigner {
    type Error = SignerError;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        let alloy_address = AlloySigner::address(self.upstream_signer());
        Ok(Address::from_bytes(alloy_address.into_array()))
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        let signature = AlloySigner::sign_message(self.upstream_signer(), message)
            .await
            .map_err(|error| SignerError::from_alloy_signer(&error))?;
        Ok(alloy_signature_to_hex(&signature)?)
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Err(SignerError::ProviderRequired {
            method: "sign_transaction",
        })
    }

    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        #[cfg(feature = "eip712")]
        {
            let typed =
                cow_typed_data_payload_to_alloy(payload).map_err(SignerError::Validation)?;
            let signature = AlloySigner::sign_dynamic_typed_data(self.upstream_signer(), &typed)
                .await
                .map_err(|error| SignerError::from_alloy_signer(&error))?;
            Ok(alloy_signature_to_hex(&signature)?)
        }

        #[cfg(not(feature = "eip712"))]
        {
            let _ = payload;
            Err(SignerError::Unsupported(
                "sign_typed_data_payload requires the eip712 feature",
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
                .map_err(SignerError::Validation)?;
            let signature = AlloySigner::sign_dynamic_typed_data(self.upstream_signer(), &typed)
                .await
                .map_err(|error| SignerError::from_alloy_signer(&error))?;
            Ok(alloy_signature_to_hex(&signature)?)
        }

        #[cfg(not(feature = "eip712"))]
        {
            let _ = (domain, fields, value_json);
            Err(SignerError::Unsupported(
                "sign_typed_data requires the eip712 feature",
            ))
        }
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Err(SignerError::ProviderRequired {
            method: "send_transaction",
        })
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Err(SignerError::ProviderRequired {
            method: "estimate_gas",
        })
    }
}

#[cfg(test)]
mod tests {
    use cow_sdk_core::{Signer as _, SupportedChainId};

    use super::*;

    const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

    #[tokio::test]
    async fn get_address_returns_signer_address() {
        let signer = test_signer();

        assert_eq!(
            signer.get_address().await.unwrap().to_hex_string(),
            "0x70997970c51812dc3a010c7d01b50e0d17dc79c8"
        );
    }

    #[tokio::test]
    async fn transaction_methods_return_provider_required() {
        let signer = test_signer();
        let tx = TransactionRequest::default();

        assert!(matches!(
            signer.sign_transaction(&tx).await,
            Err(SignerError::ProviderRequired {
                method: "sign_transaction"
            })
        ));
        assert!(matches!(
            signer.send_transaction(&tx).await,
            Err(SignerError::ProviderRequired {
                method: "send_transaction"
            })
        ));
        assert!(matches!(
            signer.estimate_gas(&tx).await,
            Err(SignerError::ProviderRequired {
                method: "estimate_gas"
            })
        ));
    }

    fn test_signer() -> LocalAlloyKeystoreSigner {
        LocalAlloyKeystoreSigner::builder()
            .private_key(TEST_KEY)
            .unwrap()
            .chain_id(SupportedChainId::Sepolia)
            .build()
            .unwrap()
    }
}
