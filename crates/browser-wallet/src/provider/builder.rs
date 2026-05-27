use std::{cell::RefCell, fmt, rc::Rc};

use cow_sdk_core::Redacted;

use crate::{BrowserWalletError, EventLog, WalletSession};

use super::{Eip1193Provider, Eip1193Transport, Origin};

/// Trust-aware builder for typed EIP-1193 providers.
///
/// Providers discovered through EIP-6963 should be built with a detected
/// origin supplied by the discovery flow. Anonymous providers must opt in
/// through [`Self::with_trusted_origin`] before construction succeeds.
pub struct Eip1193ProviderBuilder {
    transport: Rc<dyn Eip1193Transport>,
    detected_origin: Option<Origin>,
    trusted_origins: Vec<Origin>,
}

impl fmt::Debug for Eip1193ProviderBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Eip1193ProviderBuilder")
            .field("wallet_label", &self.transport.label())
            .field("detected_origin", &self.detected_origin)
            .field("trusted_origins", &self.trusted_origins)
            .finish_non_exhaustive()
    }
}

impl Eip1193ProviderBuilder {
    /// Creates a provider builder from one typed EIP-1193 transport.
    #[must_use]
    pub fn new<T>(transport: T) -> Self
    where
        T: Eip1193Transport + 'static,
    {
        Self::from_shared(Rc::new(transport))
    }

    pub(crate) fn from_shared(transport: Rc<dyn Eip1193Transport>) -> Self {
        Self {
            transport,
            detected_origin: None,
            trusted_origins: Vec::new(),
        }
    }

    pub(crate) fn with_detected_origin(mut self, origin: Origin) -> Self {
        self.detected_origin = Some(origin);
        self
    }

    /// Adds an explicitly reviewed origin for an anonymous EIP-1193 provider.
    #[must_use]
    pub fn with_trusted_origin(mut self, origin: Origin) -> Self {
        self.trusted_origins.push(origin);
        self
    }

    /// Builds a typed EIP-1193 provider.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserWalletError::UntrustedProviderOrigin`] when the
    /// provider was not discovered through EIP-6963 and no explicit trusted
    /// origin was supplied.
    pub fn build(self) -> Result<Eip1193Provider, BrowserWalletError> {
        let events = EventLog::default();
        let session = Rc::new(RefCell::new(WalletSession::new(
            false,
            None,
            Vec::new(),
            None,
            self.transport.label().to_owned(),
        )));
        self.build_with_session(session, events)
    }

    pub(crate) fn build_with_session(
        self,
        session: Rc<RefCell<WalletSession>>,
        events: EventLog,
    ) -> Result<Eip1193Provider, BrowserWalletError> {
        let origin = self.trusted_origin()?;
        {
            let mut session_state = session.borrow_mut();
            self.transport
                .label()
                .clone_into(&mut session_state.wallet_label);
        }
        Ok(Eip1193Provider::new(
            self.transport,
            session,
            events,
            origin,
        ))
    }

    fn trusted_origin(&self) -> Result<Option<Origin>, BrowserWalletError> {
        if let Some(origin) = &self.detected_origin {
            return Ok(Some(origin.clone()));
        }

        if let Some(origin) = self.trusted_origins.first() {
            warn_wallet_origin(origin, true);
            return Ok(Some(origin.clone()));
        }

        warn_anonymous_wallet_origin();
        Err(BrowserWalletError::UntrustedProviderOrigin {
            origin: Redacted::new("<anonymous>".to_owned()),
        })
    }
}

#[cfg(feature = "tracing")]
fn warn_wallet_origin(origin: &Origin, allowed: bool) {
    tracing::warn!(
        target: "cow_sdk::trust",
        origin = ?Redacted::new(origin.as_str().to_owned()),
        allowed,
        "non-discovered EIP-1193 provider origin evaluated"
    );
}

#[cfg(not(feature = "tracing"))]
const fn warn_wallet_origin(_origin: &Origin, _allowed: bool) {}

#[cfg(feature = "tracing")]
fn warn_anonymous_wallet_origin() {
    tracing::warn!(
        target: "cow_sdk::trust",
        origin = ?Redacted::new("<anonymous>".to_owned()),
        allowed = false,
        "anonymous EIP-1193 provider rejected"
    );
}

#[cfg(not(feature = "tracing"))]
const fn warn_anonymous_wallet_origin() {}
