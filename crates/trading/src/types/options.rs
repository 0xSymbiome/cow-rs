use std::{fmt, sync::Arc};

use cow_sdk_orderbook::OrderbookClient;

/// Options stored on [`crate::TradingSdk`] that do not belong in trader defaults.
#[derive(Clone, Default)]
pub struct TradingSdkOptions {
    order_book_api: Option<Arc<dyn OrderbookClient>>,
}

impl fmt::Debug for TradingSdkOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TradingSdkOptions")
            .field("order_book_api", &self.order_book_api.is_some())
            .finish()
    }
}

impl TradingSdkOptions {
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

    /// Returns the injected orderbook client, if one is configured.
    #[must_use]
    pub fn orderbook_client(&self) -> Option<Arc<dyn OrderbookClient>> {
        self.order_book_api.clone()
    }
}
