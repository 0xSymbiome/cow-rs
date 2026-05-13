use std::{cell::RefCell, rc::Rc};

use async_trait::async_trait;
use serde_json::Value;

use crate::{BrowserWalletError, EventLog, WalletSession, events::WalletRuntimeBindingHandle};

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait(?Send))]
/// Transport seam for typed EIP-1193 browser-wallet requests.
///
/// Implementors are responsible for method dispatch, request serialization, and optional session
/// listener attachment. The public SDK surface remains typed at the provider and signer layers,
/// while browser-runtime interop details stay private to the leaf crate.
pub trait Eip1193Transport {
    /// Returns the human-readable wallet label for session and event reporting.
    fn label(&self) -> &str;
    /// Executes one wallet request and returns the decoded JSON result.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserWalletError`] when the wallet rejects the request, reports an RPC error, or
    /// returns data that cannot be represented as JSON.
    async fn request(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, BrowserWalletError>;

    /// Optionally attaches runtime-native session listeners for provider-emitted events.
    fn attach_session_sync(
        &self,
        _session: Rc<RefCell<WalletSession>>,
        _events: EventLog,
    ) -> Option<WalletRuntimeBindingHandle> {
        None
    }
}
