//! [`MockOrderbook`]: an in-memory [`OrderbookClient`] double that returns canned
//! responses, registers orders for lookup, records requests, and injects
//! failures.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use async_trait::async_trait;
use cow_sdk_core::{ApiContext, AppDataHash, CowEnv, OrderUid, SupportedChainId};
use cow_sdk_orderbook::{
    Order, OrderCancellations, OrderCreation, OrderQuoteRequest, OrderQuoteResponse,
    OrderbookClient, OrderbookError,
};

use crate::{defaults, error::OrderbookFailure};

/// A recording, canned-response [`OrderbookClient`] double.
///
/// Cloning shares one backing store, so a clone injected into a `Trading`
/// client and a clone kept for assertions observe the same recorded calls.
#[derive(Clone, Debug)]
pub struct MockOrderbook {
    context: ApiContext,
    inner: Arc<Mutex<Inner>>,
}

#[derive(Debug)]
struct Inner {
    quote: OrderQuoteResponse,
    order_uid: OrderUid,
    orders: Vec<Order>,
    fail_quote: Option<OrderbookFailure>,
    fail_send: Option<OrderbookFailure>,
    calls: OrderbookCalls,
}

/// A snapshot of what a [`MockOrderbook`] was asked to do.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct OrderbookCalls {
    /// Requests passed to [`OrderbookClient::quote`].
    pub quote_requests: Vec<OrderQuoteRequest>,
    /// Orders passed to [`OrderbookClient::send_order`].
    pub sent_orders: Vec<OrderCreation>,
    /// Cancellations passed to [`OrderbookClient::send_cancellations`].
    pub cancellations: Vec<OrderCancellations>,
    /// App-data uploads passed to [`OrderbookClient::upload_app_data`].
    pub uploads: Vec<(AppDataHash, String)>,
}

impl MockOrderbook {
    /// An orderbook for `chain` (production environment) with canned defaults.
    #[must_use]
    pub fn new(chain: SupportedChainId) -> Self {
        Self::builder(chain).build()
    }

    /// Starts a builder for `chain` to configure responses and failures.
    #[must_use]
    pub fn builder(chain: SupportedChainId) -> MockOrderbookBuilder {
        MockOrderbookBuilder::new(chain)
    }

    /// A snapshot of the calls recorded so far.
    ///
    /// Every request the double received is recorded regardless of the response:
    /// a canned success and an injected failure both leave the request in the
    /// log, so an error-path test can assert the call was attempted.
    #[must_use]
    pub fn recorded(&self) -> OrderbookCalls {
        self.lock().calls.clone()
    }

    /// Registers `order` so [`OrderbookClient::order`] resolves its UID.
    pub fn push_order(&self, order: Order) {
        self.lock().orders.push(order);
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Consuming builder for [`MockOrderbook`].
#[derive(Debug)]
pub struct MockOrderbookBuilder {
    chain: SupportedChainId,
    env: CowEnv,
    quote: OrderQuoteResponse,
    order_uid: OrderUid,
    orders: Vec<Order>,
    fail_quote: Option<OrderbookFailure>,
    fail_send: Option<OrderbookFailure>,
}

impl MockOrderbookBuilder {
    fn new(chain: SupportedChainId) -> Self {
        Self {
            chain,
            env: CowEnv::Prod,
            quote: defaults::quote(),
            order_uid: defaults::order_uid(),
            orders: Vec::new(),
            fail_quote: None,
            fail_send: None,
        }
    }

    /// Sets the environment carried in the client context (default `Prod`).
    #[must_use]
    pub const fn env(mut self, env: CowEnv) -> Self {
        self.env = env;
        self
    }

    /// Sets the response [`OrderbookClient::quote`] returns.
    #[must_use]
    pub fn quote(mut self, quote: OrderQuoteResponse) -> Self {
        self.quote = quote;
        self
    }

    /// Sets the UID [`OrderbookClient::send_order`] returns.
    #[must_use]
    pub const fn order_uid(mut self, order_uid: OrderUid) -> Self {
        self.order_uid = order_uid;
        self
    }

    /// Registers `order` for [`OrderbookClient::order`] lookup.
    #[must_use]
    pub fn order(mut self, order: Order) -> Self {
        self.orders.push(order);
        self
    }

    /// Makes [`OrderbookClient::quote`] fail with `failure`.
    #[must_use]
    pub fn fail_quote(mut self, failure: OrderbookFailure) -> Self {
        self.fail_quote = Some(failure);
        self
    }

    /// Makes [`OrderbookClient::send_order`] fail with `failure`.
    #[must_use]
    pub fn fail_send(mut self, failure: OrderbookFailure) -> Self {
        self.fail_send = Some(failure);
        self
    }

    /// Builds the orderbook.
    #[must_use]
    pub fn build(self) -> MockOrderbook {
        MockOrderbook {
            context: ApiContext::new(self.chain, self.env),
            inner: Arc::new(Mutex::new(Inner {
                quote: self.quote,
                order_uid: self.order_uid,
                orders: self.orders,
                fail_quote: self.fail_quote,
                fail_send: self.fail_send,
                calls: OrderbookCalls::default(),
            })),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl OrderbookClient for MockOrderbook {
    fn context(&self) -> &ApiContext {
        &self.context
    }

    async fn quote(
        &self,
        request: &OrderQuoteRequest,
    ) -> Result<OrderQuoteResponse, OrderbookError> {
        let mut guard = self.lock();
        guard.calls.quote_requests.push(request.clone());
        if let Some(failure) = &guard.fail_quote {
            return Err(failure.to_error());
        }
        Ok(guard.quote.clone())
    }

    async fn send_order(&self, request: &OrderCreation) -> Result<OrderUid, OrderbookError> {
        let mut guard = self.lock();
        guard.calls.sent_orders.push(request.clone());
        if let Some(failure) = &guard.fail_send {
            return Err(failure.to_error());
        }
        Ok(guard.order_uid)
    }

    async fn send_cancellations(&self, request: &OrderCancellations) -> Result<(), OrderbookError> {
        self.lock().calls.cancellations.push(request.clone());
        Ok(())
    }

    async fn order(&self, order_uid: &OrderUid) -> Result<Order, OrderbookError> {
        self.lock()
            .orders
            .iter()
            .find(|order| &order.uid == order_uid)
            .cloned()
            .ok_or_else(crate::error::order_not_found)
    }

    async fn upload_app_data(
        &self,
        app_data_hash: &AppDataHash,
        full_app_data: &str,
    ) -> Result<(), OrderbookError> {
        self.lock()
            .calls
            .uploads
            .push((*app_data_hash, full_app_data.to_owned()));
        Ok(())
    }
}
