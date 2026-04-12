//! Typed wallet session and event-log state.

use std::{cell::RefCell, rc::Rc};

use cow_sdk_core::{Address, ChainId};
use serde::{Deserialize, Serialize};

/// Current wallet session state tracked by the browser-wallet integration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletSession {
    /// Whether the wallet is currently connected.
    pub connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Last known connected chain id.
    pub chain_id: Option<ChainId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Accounts currently exposed by the wallet.
    pub accounts: Vec<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Currently selected account, when one is available.
    pub selected_account: Option<Address>,
    /// Human-readable wallet label used by the session and event stream.
    pub wallet_label: String,
}

/// Typed wallet events emitted by provider requests and provider-driven session updates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WalletEvent {
    /// A wallet request has started.
    RequestStarted {
        /// RPC method being requested.
        method: String,
    },
    /// A wallet request completed successfully.
    RequestSucceeded {
        /// RPC method that succeeded.
        method: String,
    },
    /// A wallet request failed.
    RequestFailed {
        /// RPC method that failed.
        method: String,
        /// Rendered failure message.
        message: String,
    },
    /// The provider reported a connect event.
    Connected {
        #[serde(skip_serializing_if = "Option::is_none")]
        /// Chain id supplied by the provider, when present.
        chain_id: Option<ChainId>,
    },
    /// The provider reported a disconnect event.
    Disconnected {
        #[serde(skip_serializing_if = "Option::is_none")]
        /// Provider-supplied disconnect message, when present.
        message: Option<String>,
    },
    /// The provider reported an account list change.
    AccountsChanged {
        /// Updated wallet account list.
        accounts: Vec<Address>,
    },
    /// The provider reported a chain change.
    ChainChanged {
        /// Updated connected chain id.
        chain_id: ChainId,
    },
    /// The normalized wallet session changed.
    SessionUpdated {
        /// Previous wallet session snapshot.
        previous: WalletSession,
        /// Current wallet session snapshot.
        current: WalletSession,
    },
}

/// In-memory event log for deterministic session and request observation.
#[derive(Debug, Clone, Default)]
pub struct EventLog(Rc<RefCell<Vec<WalletEvent>>>);

impl EventLog {
    /// Appends one event to the log.
    pub fn push(&self, event: WalletEvent) {
        self.0.borrow_mut().push(event);
    }

    /// Returns a cloned snapshot of all currently buffered events.
    #[must_use]
    pub fn snapshot(&self) -> Vec<WalletEvent> {
        self.0.borrow().clone()
    }

    /// Drains and returns all currently buffered events.
    #[must_use]
    pub fn take(&self) -> Vec<WalletEvent> {
        self.0.borrow_mut().drain(..).collect()
    }
}

#[doc(hidden)]
pub trait WalletRuntimeBinding {}

#[doc(hidden)]
pub type WalletRuntimeBindingHandle = Rc<dyn WalletRuntimeBinding>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WalletProviderEvent {
    AccountsChanged { accounts: Vec<Address> },
    ChainChanged { chain_id: ChainId },
    Connected { chain_id: Option<ChainId> },
    Disconnected { message: Option<String> },
}

pub(crate) fn update_wallet_session<F>(
    session: &Rc<RefCell<WalletSession>>,
    events: &EventLog,
    explicit_event: Option<WalletEvent>,
    updater: F,
) where
    F: FnOnce(&mut WalletSession),
{
    let previous = session.borrow().clone();
    {
        let mut current = session.borrow_mut();
        updater(&mut current);
    }
    let current = session.borrow().clone();

    if let Some(event) = explicit_event {
        events.push(event);
    }
    if previous.chain_id != current.chain_id
        && let Some(chain_id) = current.chain_id
    {
        events.push(WalletEvent::ChainChanged { chain_id });
    }
    if previous.accounts != current.accounts {
        events.push(WalletEvent::AccountsChanged {
            accounts: current.accounts.clone(),
        });
    }
    if previous != current {
        events.push(WalletEvent::SessionUpdated { previous, current });
    }
}

pub(crate) fn apply_provider_event(
    session: &Rc<RefCell<WalletSession>>,
    events: &EventLog,
    provider_event: WalletProviderEvent,
) {
    match provider_event {
        WalletProviderEvent::AccountsChanged { accounts } => {
            update_wallet_session(session, events, None, |session| {
                session.connected = !accounts.is_empty();
                session.accounts.clone_from(&accounts);
                session.selected_account = accounts.first().cloned();
            });
        }
        WalletProviderEvent::ChainChanged { chain_id } => {
            update_wallet_session(session, events, None, |session| {
                session.chain_id = Some(chain_id);
            });
        }
        WalletProviderEvent::Connected { chain_id } => {
            update_wallet_session(
                session,
                events,
                Some(WalletEvent::Connected { chain_id }),
                |session| {
                    session.connected = true;
                    if let Some(chain_id) = chain_id {
                        session.chain_id = Some(chain_id);
                    }
                },
            );
        }
        WalletProviderEvent::Disconnected { message } => {
            update_wallet_session(
                session,
                events,
                Some(WalletEvent::Disconnected { message }),
                |session| {
                    session.connected = false;
                    session.chain_id = None;
                    session.accounts.clear();
                    session.selected_account = None;
                },
            );
        }
    }
}
