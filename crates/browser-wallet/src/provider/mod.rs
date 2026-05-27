//! Typed EIP-1193 provider bridge and `Provider` implementation.
//!
//! This module keeps browser-wallet request execution typed and local to the leaf crate. It does
//! not expose a generic raw wallet-RPC passthrough beyond the transport seam used by the typed
//! provider and signer adapters.

mod builder;
mod origin;
mod provider_impl;
mod signing_provider_impl;
mod transport;

use std::{cell::RefCell, fmt, rc::Rc};

use serde_json::Value;

use cow_sdk_core::{Address, ChainId};

use crate::{
    BrowserWalletError, EventLog, WalletEvent, WalletSession,
    events::{WalletRuntimeBindingHandle, update_wallet_session},
};

use self::provider_impl::parse_address_array;

pub(crate) use self::provider_impl::{
    hex_quantity, parse_chain_id_value, parse_quantity_to_decimal, transaction_to_rpc,
};
pub use self::{builder::Eip1193ProviderBuilder, origin::Origin, transport::Eip1193Transport};

/// Typed browser-wallet provider that implements [`cow_sdk_core::Provider`]
/// and [`cow_sdk_core::SigningProvider`].
#[derive(Clone)]
pub struct Eip1193Provider {
    transport: Rc<dyn Eip1193Transport>,
    session: Rc<RefCell<WalletSession>>,
    events: EventLog,
    origin: Option<Origin>,
    _runtime_binding: Option<WalletRuntimeBindingHandle>,
}

impl fmt::Debug for Eip1193Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let session = self.session.borrow().clone();
        f.debug_struct("Eip1193Provider")
            .field("wallet_label", &session.wallet_label)
            .field("session", &session)
            .field("origin", &self.origin)
            .finish_non_exhaustive()
    }
}

impl Eip1193Provider {
    pub(crate) fn new(
        transport: Rc<dyn Eip1193Transport>,
        session: Rc<RefCell<WalletSession>>,
        events: EventLog,
        origin: Option<Origin>,
    ) -> Self {
        let runtime_binding = transport.attach_session_sync(session.clone(), events.clone());
        Self {
            transport,
            session,
            events,
            origin,
            _runtime_binding: runtime_binding,
        }
    }

    /// Returns the current normalized wallet session snapshot.
    #[must_use]
    pub fn session(&self) -> WalletSession {
        self.session.borrow().clone()
    }

    pub(crate) fn events(&self) -> EventLog {
        self.events.clone()
    }

    /// Returns the reviewed provider origin label, if one was captured at construction.
    #[must_use]
    pub const fn origin(&self) -> Option<&Origin> {
        self.origin.as_ref()
    }

    /// Returns the currently selected wallet account, when available.
    #[must_use]
    pub fn selected_account(&self) -> Option<Address> {
        self.session.borrow().selected_account
    }

    /// Clears the cached wallet session state while preserving the wallet label.
    #[must_use]
    pub fn reset_session(&self) -> WalletSession {
        let wallet_label = self.session.borrow().wallet_label.clone();
        self.update_session(move |session| {
            *session = WalletSession::new(false, None, Vec::new(), None, wallet_label);
        });
        self.session()
    }

    /// Queries wallet accounts and updates the cached session state.
    ///
    /// When `interactive` is `true`, this uses `eth_requestAccounts` and may trigger a wallet
    /// authorization prompt. When it is `false`, this uses `eth_accounts` and performs a passive
    /// account lookup only.
    ///
    /// # Errors
    ///
    /// Returns an error when the wallet rejects the request or returns a malformed account list.
    pub async fn query_accounts(
        &self,
        interactive: bool,
    ) -> Result<Vec<Address>, BrowserWalletError> {
        let method = if interactive {
            "eth_requestAccounts"
        } else {
            "eth_accounts"
        };
        let value = self.request(method, None).await?;
        let accounts = parse_address_array(&value, method)?;
        self.update_session(|session| {
            session.connected = !accounts.is_empty();
            session.accounts.clone_from(&accounts);
            session.selected_account = accounts.first().copied();
        });
        Ok(accounts)
    }

    /// Queries the connected chain id and updates the cached session state.
    ///
    /// # Errors
    ///
    /// Returns an error when the wallet rejects `eth_chainId` or returns a malformed chain id.
    pub async fn query_chain_id(&self) -> Result<ChainId, BrowserWalletError> {
        let value = self.request("eth_chainId", None).await?;
        let chain_id = parse_chain_id_value(&value, "eth_chainId")?;
        self.update_session(|session| {
            session.chain_id = Some(chain_id);
        });
        Ok(chain_id)
    }

    pub(crate) async fn request(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, BrowserWalletError> {
        self.events.push(WalletEvent::RequestStarted {
            method: method.to_owned(),
        });
        match self.transport.request(method, params).await {
            Ok(value) => {
                self.events.push(WalletEvent::RequestSucceeded {
                    method: method.to_owned(),
                });
                Ok(value)
            }
            Err(error) => {
                self.events.push(WalletEvent::RequestFailed {
                    method: method.to_owned(),
                    message: error.to_string(),
                });
                Err(error)
            }
        }
    }

    pub(crate) fn update_session<F>(&self, updater: F)
    where
        F: FnOnce(&mut WalletSession),
    {
        update_wallet_session(&self.session, &self.events, None, updater);
    }
}
