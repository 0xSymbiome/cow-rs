use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};
use serde_json::json;

use cow_sdk_core::{Address, ChainId, SupportedChainId};

use crate::{
    BrowserWalletError, Eip1193Provider, Eip1193Signer, Eip1193Transport, EventLog, WalletSession,
    provider::{Eip1193Provider as ProviderImpl, hex_quantity},
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectedWalletInfo {
    pub provider_label: String,
    pub is_meta_mask: bool,
    pub is_coinbase_wallet: bool,
    pub is_rabby: bool,
}

#[derive(Clone)]
pub struct BrowserWallet {
    provider: Eip1193Provider,
    injected_info: Option<InjectedWalletInfo>,
}

impl BrowserWallet {
    pub fn from_transport<T>(transport: T) -> Self
    where
        T: Eip1193Transport + 'static,
    {
        Self::from_parts(Rc::new(transport), None)
    }

    pub fn injected_info(&self) -> Option<InjectedWalletInfo> {
        self.injected_info.clone()
    }

    pub fn provider(&self) -> Eip1193Provider {
        self.provider.clone()
    }

    pub fn signer(&self) -> Eip1193Signer {
        Eip1193Signer::new(self.provider.clone(), None)
    }

    pub fn session(&self) -> WalletSession {
        self.provider.session()
    }

    pub fn account(&self) -> Option<Address> {
        self.session().selected_account
    }

    pub fn chain_id(&self) -> Option<ChainId> {
        self.session().chain_id
    }

    pub fn reset_session(&self) -> WalletSession {
        self.provider.reset_session()
    }

    pub fn take_events(&self) -> Vec<crate::WalletEvent> {
        self.provider.events().take()
    }

    pub fn events(&self) -> Vec<crate::WalletEvent> {
        self.provider.events().snapshot()
    }

    pub async fn connect(&self) -> Result<WalletSession, BrowserWalletError> {
        self.provider.query_accounts(true).await?;
        self.provider.query_chain_id().await?;
        Ok(self.session())
    }

    pub async fn request_accounts(&self) -> Result<Vec<Address>, BrowserWalletError> {
        self.provider.query_accounts(true).await
    }

    pub async fn refresh_session(&self) -> Result<WalletSession, BrowserWalletError> {
        let _ = self.provider.query_accounts(false).await?;
        let _ = self.provider.query_chain_id().await?;
        Ok(self.session())
    }

    pub async fn switch_chain(
        &self,
        chain_id: SupportedChainId,
    ) -> Result<WalletSession, BrowserWalletError> {
        self.provider
            .request(
                "wallet_switchEthereumChain",
                Some(json!([{ "chainId": hex_quantity(&u64::from(chain_id).to_string())? }])),
            )
            .await?;
        self.refresh_session().await
    }

    #[cfg(target_arch = "wasm32")]
    pub fn detect() -> Result<Option<Self>, BrowserWalletError> {
        let Some(transport) = crate::js::InjectedProviderTransport::detect()? else {
            return Ok(None);
        };
        let info = transport.info();
        Ok(Some(Self::from_parts(Rc::new(transport), Some(info))))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn detect() -> Result<Option<Self>, BrowserWalletError> {
        Ok(None)
    }

    fn from_parts(
        transport: Rc<dyn Eip1193Transport>,
        injected_info: Option<InjectedWalletInfo>,
    ) -> Self {
        let events = EventLog::default();
        let session = Rc::new(RefCell::new(WalletSession {
            wallet_label: transport.label().to_owned(),
            ..WalletSession::default()
        }));
        let provider = ProviderImpl::new(transport, session, events);
        Self {
            provider,
            injected_info,
        }
    }
}
