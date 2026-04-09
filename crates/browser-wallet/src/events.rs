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
