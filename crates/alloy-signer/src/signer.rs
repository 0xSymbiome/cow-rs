//! `Signer` implementation backed by Alloy's local private-key signer.

use std::{fmt, sync::Arc};

use alloy_signer::Signer as AlloySigner;
use alloy_signer_local::PrivateKeySigner;
use cow_sdk_core::{
    Address, Amount, ChainId, Signer, SupportedChainId, TransactionBroadcast, TransactionRequest,
    TypedDataPayload,
};

use crate::{
    builder::LocalAlloySignerBuilder,
    conversion::{alloy_signature_to_hex, cow_typed_data_payload_to_alloy},
    error::SignerError,
};

/// Alloy-backed local private-key signer for native SDK consumers.
#[derive(Clone)]
pub struct LocalAlloySigner {
    inner: Arc<PrivateKeySigner>,
    chain_id: ChainId,
}

impl LocalAlloySigner {
    /// Returns a fresh typestate builder.
    pub const fn builder() -> LocalAlloySignerBuilder {
        LocalAlloySignerBuilder::new()
    }

    /// Binds a private key (hex, with or without the `0x` prefix) to a chain in
    /// one call.
    ///
    /// The `from_x` total-input shortcut. Alloy parses the key and binds the
    /// chain in two optional steps (`FromStr`, then `Signer::with_chain_id`);
    /// this signer instead requires the chain up front — it backs the EIP-712
    /// domain — so it takes both. Reach for [`LocalAlloySigner::builder`] for raw
    /// key bytes or explicit typestate construction.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidPrivateKey`](crate::LocalAlloySignerBuilderError::InvalidPrivateKey)
    /// when the input is not a valid secp256k1 private key. The error carries no
    /// key material.
    pub fn from_private_key(
        hex: impl AsRef<str>,
        chain_id: SupportedChainId,
    ) -> Result<Self, crate::builder::LocalAlloySignerBuilderError> {
        let inner = crate::builder::parse_private_key_signer(hex.as_ref())
            .ok_or(crate::builder::LocalAlloySignerBuilderError::InvalidPrivateKey)?;
        Ok(Self::from_parts(inner, ChainId::from(chain_id)))
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

impl fmt::Debug for LocalAlloySigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalAlloySigner")
            .field("inner", &"[redacted]")
            .field("chain_id", &self.chain_id)
            .finish()
    }
}

impl Signer for LocalAlloySigner {
    type Error = SignerError;

    fn chain_id(&self) -> Option<SupportedChainId> {
        // The typestate builder binds a numeric chain at construction; report
        // it as the supported-chain hint when it is one the orderbook knows,
        // so a trading flow can fast-fail a signer/trading chain mismatch
        // before signing. A numeric chain outside the supported set opts out.
        SupportedChainId::try_from(self.chain_id).ok()
    }

    async fn address(&self) -> Result<Address, Self::Error> {
        let alloy_address = AlloySigner::address(self.upstream_signer());
        Ok(Address::from_bytes(alloy_address.into_array()))
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        let signature = AlloySigner::sign_message(self.upstream_signer(), message)
            .await
            .map_err(|error| SignerError::from_alloy_signer(&error))?;
        Ok(alloy_signature_to_hex(&signature)?)
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
