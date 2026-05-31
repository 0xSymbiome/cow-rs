use std::{fmt, sync::Arc};

use cow_sdk_orderbook::OrderbookClient;
use cow_sdk_transport_policy::TransportPolicy;

/// Options stored on [`crate::Trading`] that do not belong in trader defaults.
#[derive(Clone, Default)]
pub struct TradingOptions {
    order_book_api: Option<Arc<dyn OrderbookClient>>,
    transport_policy: Option<TransportPolicy>,
}

impl fmt::Debug for TradingOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TradingOptions")
            .field("order_book_api", &self.order_book_api.is_some())
            .field("transport_policy", &self.transport_policy)
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
    /// The injected client fixes chain and environment for orderbook-bound flows.
    #[must_use]
    pub fn with_orderbook_client(mut self, orderbook_client: Arc<dyn OrderbookClient>) -> Self {
        self.order_book_api = Some(orderbook_client);
        self
    }

    /// Returns a copy of these options with the request retry, rate-limit, and
    /// HTTP-client policy applied to the orderbook client the trading SDK
    /// builds on the default construction path.
    ///
    /// This setting governs only the orderbook client the SDK builds for
    /// itself when no client is injected. When an orderbook client is supplied
    /// through [`TradingOptions::with_orderbook_client`], that client already
    /// carries its own [`TransportPolicy`] and this value is not consulted.
    #[must_use]
    pub fn with_transport_policy(mut self, transport_policy: TransportPolicy) -> Self {
        self.transport_policy = Some(transport_policy);
        self
    }

    /// Returns the injected orderbook client, if one is configured.
    #[must_use]
    pub fn orderbook_client(&self) -> Option<Arc<dyn OrderbookClient>> {
        self.order_book_api.clone()
    }

    /// Returns the transport policy applied to the default-built orderbook
    /// client, if one is configured.
    #[must_use]
    pub fn transport_policy(&self) -> Option<TransportPolicy> {
        self.transport_policy.clone()
    }
}
