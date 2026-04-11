use std::{cell::RefCell, rc::Rc};

use cow_sdk_core::{Address, ChainId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletSession {
    pub connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<ChainId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accounts: Vec<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_account: Option<Address>,
    pub wallet_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WalletEvent {
    RequestStarted {
        method: String,
    },
    RequestSucceeded {
        method: String,
    },
    RequestFailed {
        method: String,
        message: String,
    },
    Connected {
        #[serde(skip_serializing_if = "Option::is_none")]
        chain_id: Option<ChainId>,
    },
    Disconnected {
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    AccountsChanged {
        accounts: Vec<Address>,
    },
    ChainChanged {
        chain_id: ChainId,
    },
    SessionUpdated {
        previous: WalletSession,
        current: WalletSession,
    },
}

#[derive(Clone, Default)]
pub struct EventLog(Rc<RefCell<Vec<WalletEvent>>>);

impl EventLog {
    pub fn push(&self, event: WalletEvent) {
        self.0.borrow_mut().push(event);
    }

    pub fn snapshot(&self) -> Vec<WalletEvent> {
        self.0.borrow().clone()
    }

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
                session.accounts = accounts.clone();
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
