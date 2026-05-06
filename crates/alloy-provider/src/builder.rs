//! Typestate builder for the native Alloy provider adapter.

use std::{fmt, time::Duration};

use cow_sdk_core::Redacted;
use thiserror::Error;

use crate::{client, provider::RpcAlloyProvider};

mod sealed {
    /// Typestate marker for a builder that has not selected a transport.
    pub struct TransportUnset {
        pub(super) _private: (),
    }

    /// Typestate marker for a builder configured with HTTP transport.
    pub struct HttpTransport {
        pub(super) url: cow_sdk_core::Redacted<reqwest::Url>,
    }

    pub trait SealedTransport {}

    impl SealedTransport for TransportUnset {}
    impl SealedTransport for HttpTransport {}
}

pub use sealed::{HttpTransport, TransportUnset};

/// Sealed marker trait implemented by every provider-builder transport state.
pub trait TransportState: sealed::SealedTransport {}

impl TransportState for TransportUnset {}
impl TransportState for HttpTransport {}

pub(crate) trait TransportSelected: TransportState {}

impl TransportSelected for HttpTransport {}

/// Typestate builder for [`RpcAlloyProvider`].
///
/// The generic state parameter records whether the required transport has been
/// selected. [`RpcAlloyProviderBuilder::build`] exists only for the
/// [`HttpTransport`] state.
#[must_use]
pub struct RpcAlloyProviderBuilder<T: TransportState = TransportUnset> {
    transport: T,
    timeout: Option<Duration>,
}

impl Default for RpcAlloyProviderBuilder<TransportUnset> {
    fn default() -> Self {
        Self::new()
    }
}

impl RpcAlloyProviderBuilder<TransportUnset> {
    /// Creates a builder with no transport selected.
    pub const fn new() -> Self {
        Self {
            transport: TransportUnset { _private: () },
            timeout: None,
        }
    }

    /// Selects native HTTP transport for the provider.
    ///
    /// # Errors
    ///
    /// Returns [`RpcAlloyProviderBuilderError::InvalidUrl`] if the input is
    /// not a valid URL. The invalid URL value is never echoed.
    pub fn http(
        self,
        rpc_url: impl AsRef<str>,
    ) -> Result<RpcAlloyProviderBuilder<HttpTransport>, RpcAlloyProviderBuilderError> {
        let url = reqwest::Url::parse(rpc_url.as_ref())
            .map_err(|_| RpcAlloyProviderBuilderError::InvalidUrl)?;

        Ok(RpcAlloyProviderBuilder {
            transport: HttpTransport {
                url: Redacted::new(url),
            },
            timeout: self.timeout,
        })
    }
}

impl<T: TransportState> RpcAlloyProviderBuilder<T> {
    /// Configures the HTTP client timeout used by the underlying Alloy provider.
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

impl RpcAlloyProviderBuilder<HttpTransport> {
    /// Builds an [`RpcAlloyProvider`] after HTTP transport has been selected.
    ///
    /// # Errors
    ///
    /// Returns [`RpcAlloyProviderBuilderError::TransportInit`] if the native
    /// HTTP client cannot be constructed.
    #[allow(
        clippy::unused_async,
        reason = "builder stays async-compatible with future transport setup and existing examples"
    )]
    pub async fn build(self) -> Result<RpcAlloyProvider, RpcAlloyProviderBuilderError> {
        require_selected_transport(&self.transport);
        let url = self.transport.url.into_inner();
        let mut client_builder = reqwest::Client::builder();
        if let Some(timeout) = self.timeout {
            client_builder = client_builder.timeout(timeout);
        }
        let client = client_builder.build().map_err(|error| {
            RpcAlloyProviderBuilderError::TransportInit {
                detail: Redacted::new(error.to_string()),
            }
        })?;
        let inner = client::build_http_provider(client, url.clone());

        Ok(RpcAlloyProvider::from_parts(
            inner,
            Redacted::new(url.to_string()),
        ))
    }
}

const fn require_selected_transport<T: TransportSelected>(_transport: &T) {}

impl fmt::Debug for RpcAlloyProviderBuilder<TransportUnset> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RpcAlloyProviderBuilder")
            .field("transport", &"unset")
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl fmt::Debug for RpcAlloyProviderBuilder<HttpTransport> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RpcAlloyProviderBuilder")
            .field("transport", &Redacted::new("http"))
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl fmt::Debug for TransportUnset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TransportUnset")
    }
}

impl fmt::Debug for HttpTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpTransport")
            .field("url", &self.url)
            .finish()
    }
}

/// Errors returned while constructing [`RpcAlloyProvider`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum RpcAlloyProviderBuilderError {
    /// The configured RPC URL could not be parsed.
    #[error("rpc url failed to parse")]
    InvalidUrl,
    /// The native HTTP transport stack could not be initialized.
    #[error("transport stack failed to initialize: {detail}")]
    TransportInit {
        /// Redacted initialization detail.
        detail: Redacted<String>,
    },
}
