use std::{fmt, sync::Arc};

use cow_sdk_orderbook::OrderbookClient;

/// Options stored on [`crate::Trading`] that do not belong in trader defaults.
#[derive(Clone, Default)]
pub struct TradingOptions {
    order_book_api: Option<Arc<dyn OrderbookClient>>,
}

impl fmt::Debug for TradingOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TradingOptions")
            .field("order_book_api", &self.order_book_api.is_some())
            .finish()
    }
}

impl TradingOptions {
    /// Creates an empty options bundle.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy of these options with an injected orderbook client.
    ///
    /// The injected client fixes chain and environment for orderbook-bound
    /// flows and carries its own [`TransportPolicy`] (retry, rate-limit, and
    /// HTTP-client tuning). Configure that resilience on the client before
    /// injecting it — build it through
    /// [`OrderbookApi::builder().transport_policy(...)`] — rather than on the
    /// trading options. On the default construction path (no client injected),
    /// the SDK builds an orderbook client with the standard
    /// [`TransportPolicy::default_orderbook`] policy.
    ///
    /// [`TransportPolicy`]: cow_sdk_transport_policy::TransportPolicy
    /// [`OrderbookApi::builder().transport_policy(...)`]: cow_sdk_orderbook::OrderbookApiBuilder::transport_policy
    /// [`TransportPolicy::default_orderbook`]: cow_sdk_transport_policy::TransportPolicy::default_orderbook
    #[must_use]
    pub fn with_orderbook_client(mut self, orderbook_client: Arc<dyn OrderbookClient>) -> Self {
        self.order_book_api = Some(orderbook_client);
        self
    }

    /// Returns the injected orderbook client, if one is configured.
    #[must_use]
    pub fn orderbook_client(&self) -> Option<Arc<dyn OrderbookClient>> {
        self.order_book_api.clone()
    }
}
