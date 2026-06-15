use serde_json::json;

use cow_sdk_core::SupportedChainId;

use crate::{BrowserWalletError, WalletSession, provider::hex_quantity};

use super::{BrowserWallet, WalletChainChange, WalletChainChangeKind, WalletChainParameters};

impl BrowserWallet {
    /// Ensures the wallet currently reports one expected chain id.
    ///
    /// # Errors
    ///
    /// Returns an error when the wallet rejects `eth_chainId`, reports a
    /// malformed chain id, or is connected to a different chain than
    /// `chain_id`.
    pub async fn ensure_chain(
        &self,
        chain_id: SupportedChainId,
    ) -> Result<WalletSession, BrowserWalletError> {
        let session_chain_id = self.provider.query_chain_id().await?;
        let expected_chain_id = u64::from(chain_id);
        if session_chain_id != expected_chain_id {
            return Err(BrowserWalletError::SessionChainMismatch {
                expected_chain_id,
                session_chain_id,
            });
        }
        Ok(self.session())
    }

    /// Switches to a supported chain and returns the refreshed session snapshot.
    ///
    /// The returned session must report the requested chain after the switch
    /// request completes.
    ///
    /// # Errors
    ///
    /// Returns an error when the wallet rejects the switch request, does not support the method,
    /// or reports that the chain has not been added.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?chain_id,
                method = "browser_wallet.switch_chain",
            ),
        ),
    )]
    pub async fn switch_chain(
        &self,
        chain_id: SupportedChainId,
    ) -> Result<WalletSession, BrowserWalletError> {
        self.switch_chain_request(chain_id).await?;
        self.refresh_session_and_ensure_chain(chain_id).await
    }

    /// Adds one typed chain configuration through `wallet_addEthereumChain`.
    ///
    /// # Errors
    ///
    /// Returns an error when the chain parameters are invalid, when the wallet rejects the add
    /// request, or when the refreshed session cannot be loaded afterwards.
    pub async fn add_chain(
        &self,
        parameters: &WalletChainParameters,
    ) -> Result<WalletChainChange, BrowserWalletError> {
        self.add_chain_request(parameters).await?;
        let session = self.refresh_session().await?;
        Ok(WalletChainChange::new(
            parameters.chain_id,
            WalletChainChangeKind::Added,
            session,
        ))
    }

    /// Switches to a chain, or adds it first when the wallet reports it is not present.
    ///
    /// Successful switch results are returned only after the refreshed session
    /// reports the requested chain.
    ///
    /// # Errors
    ///
    /// Returns an error when the switch request fails for reasons other than chain absence, when
    /// the typed add-chain request is invalid, when the wallet rejects either request, or when the
    /// refreshed session cannot be loaded afterwards.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?parameters.chain_id,
                method = "browser_wallet.switch_or_add_chain",
            ),
        ),
    )]
    pub async fn switch_or_add_chain(
        &self,
        parameters: &WalletChainParameters,
    ) -> Result<WalletChainChange, BrowserWalletError> {
        match self.switch_chain_request(parameters.chain_id).await {
            Ok(()) => {
                let session = self
                    .refresh_session_and_ensure_chain(parameters.chain_id)
                    .await?;
                Ok(WalletChainChange::new(
                    parameters.chain_id,
                    WalletChainChangeKind::Switched,
                    session,
                ))
            }
            Err(BrowserWalletError::ChainNotAdded { chain_id, .. })
                if chain_id == Some(u64::from(parameters.chain_id)) =>
            {
                self.add_chain_request(parameters).await?;
                self.switch_chain_request(parameters.chain_id).await?;
                let session = self
                    .refresh_session_and_ensure_chain(parameters.chain_id)
                    .await?;
                Ok(WalletChainChange::new(
                    parameters.chain_id,
                    WalletChainChangeKind::AddedThenSwitched,
                    session,
                ))
            }
            Err(error) => Err(error),
        }
    }

    async fn refresh_session_and_ensure_chain(
        &self,
        chain_id: SupportedChainId,
    ) -> Result<WalletSession, BrowserWalletError> {
        let _ = self.refresh_session().await?;
        self.ensure_chain(chain_id).await
    }

    async fn switch_chain_request(
        &self,
        chain_id: SupportedChainId,
    ) -> Result<(), BrowserWalletError> {
        self.provider
            .request(
                "wallet_switchEthereumChain",
                Some(json!([{ "chainId": hex_quantity(&u64::from(chain_id).to_string())? }])),
            )
            .await
            .map(|_| ())
    }

    async fn add_chain_request(
        &self,
        parameters: &WalletChainParameters,
    ) -> Result<(), BrowserWalletError> {
        self.provider
            .request(
                "wallet_addEthereumChain",
                Some(json!([parameters.rpc_payload()?])),
            )
            .await
            .map(|_| ())
    }
}
