//! PROTOTYPE / SIMULATION — feature `fluent-preview`, off by default.
//!
//! A fluent, typed `swap` lifecycle façade over the existing [`Trading`]
//! methods, built to evaluate the trading-typestate-gap strategy
//! (`.local/.devdocs/.progress/trading/`). It adds **no** protocol logic:
//! [`SwapBuilder::quote`] delegates to [`Trading::quote_results`] and
//! [`QuotedSwap::submit`] delegates to [`Trading::post_swap_order_from_quote`].
//!
//! Two design wins this proves out:
//! 1. Each token has its own **named** setter, so the sell and buy addresses
//!    cannot be transposed the way the positional `TradeParameters::new(kind,
//!    sell, buy, amount)` constructor allows.
//! 2. The quote is carried forward into `submit`, so the submitted order is the
//!    exact quoted order (no re-quote drift).
//!
//! Productionization notes (NOT done here, deliberately): seal the `Set`/`Unset`
//! markers with private tuple fields per ADR 0013, add the trybuild witnesses,
//! and decide the owner/signer ergonomics question documented in the dossier.

use std::marker::PhantomData;
use std::sync::Arc;

use cow_sdk_core::{Address, Amount, OrderKind, Signer, SignerError};

use super::{Trading, TradingBuilder};
use crate::{
    OrderbookClient, OrderPostingResult, QuoteResults, TradeAdvancedSettings, TradeParameters,
    TradingError,
};

/// Ergonomics: accept an owned orderbook client and wrap it internally, so
/// consumers never write `Arc::new(...)`.
///
/// Internal storage stays `Arc<dyn OrderbookClient>` (type-erased + `Clone`),
/// which is what keeps `Trading` a single concrete type (no `Trading<O>` generic
/// leak), keeps the wasm-bindgen surface generic-free, and lets the same fluent
/// chain run native (reqwest) or in a browser (fetch). The `Arc` is an internal
/// implementation detail, not a consumer-facing tax — this method hides it.
impl<C, A> TradingBuilder<C, A> {
    /// Injects an orderbook client by value — no `Arc::new(...)` at the call site.
    #[must_use]
    pub fn orderbook(self, client: impl OrderbookClient + 'static) -> Self {
        self.orderbook_client(Arc::new(client))
    }
}

/// Typestate marker: a required field has not been supplied yet.
#[derive(Debug, Clone, Copy)]
pub struct Unset;

/// Typestate marker: a required field has been supplied.
#[derive(Debug, Clone, Copy)]
pub struct Set;

impl Trading {
    /// Opens a fluent, typed swap chain.
    ///
    /// Set the sell token, buy token, and an amount (via [`SwapBuilder::sell_amount`]
    /// or [`SwapBuilder::buy_amount`]); only then does [`SwapBuilder::quote`] compile.
    #[must_use]
    pub const fn swap(&self) -> SwapBuilder<'_> {
        SwapBuilder {
            trading: self,
            kind: OrderKind::Sell,
            sell_token: None,
            buy_token: None,
            amount: None,
            owner: None,
            slippage_bps: None,
            receiver: None,
            valid_for: None,
            valid_to: None,
            partially_fillable: false,
            advanced: None,
            _state: PhantomData,
        }
    }
}

/// Fluent, typed builder for a swap quote. Reached through [`Trading::swap`].
///
/// The three type parameters track whether the sell token, buy token, and amount
/// have been supplied. [`SwapBuilder::quote`] exists only on
/// `SwapBuilder<Set, Set, Set>`.
#[derive(Debug, Clone)]
pub struct SwapBuilder<'t, SellToken = Unset, BuyToken = Unset, AmountState = Unset> {
    trading: &'t Trading,
    kind: OrderKind,
    sell_token: Option<Address>,
    buy_token: Option<Address>,
    amount: Option<Amount>,
    owner: Option<Address>,
    slippage_bps: Option<u32>,
    receiver: Option<Address>,
    valid_for: Option<u32>,
    valid_to: Option<u32>,
    partially_fillable: bool,
    advanced: Option<TradeAdvancedSettings>,
    _state: PhantomData<(SellToken, BuyToken, AmountState)>,
}

impl<'t, SellToken, BuyToken, AmountState> SwapBuilder<'t, SellToken, BuyToken, AmountState> {
    fn cast<NextSell, NextBuy, NextAmount>(self) -> SwapBuilder<'t, NextSell, NextBuy, NextAmount> {
        SwapBuilder {
            trading: self.trading,
            kind: self.kind,
            sell_token: self.sell_token,
            buy_token: self.buy_token,
            amount: self.amount,
            owner: self.owner,
            slippage_bps: self.slippage_bps,
            receiver: self.receiver,
            valid_for: self.valid_for,
            valid_to: self.valid_to,
            partially_fillable: self.partially_fillable,
            advanced: self.advanced,
            _state: PhantomData,
        }
    }

    /// Sets the sell-token address. Named so it cannot be transposed with the buy token.
    #[must_use]
    pub fn sell_token(mut self, sell_token: Address) -> SwapBuilder<'t, Set, BuyToken, AmountState> {
        self.sell_token = Some(sell_token);
        self.cast()
    }

    /// Sets the buy-token address. Named so it cannot be transposed with the sell token.
    #[must_use]
    pub fn buy_token(mut self, buy_token: Address) -> SwapBuilder<'t, SellToken, Set, AmountState> {
        self.buy_token = Some(buy_token);
        self.cast()
    }

    /// Sets an exact sell amount (`OrderKind::Sell`).
    #[must_use]
    pub fn sell_amount(mut self, amount: Amount) -> SwapBuilder<'t, SellToken, BuyToken, Set> {
        self.kind = OrderKind::Sell;
        self.amount = Some(amount);
        self.cast()
    }

    /// Sets an exact buy amount (`OrderKind::Buy`).
    #[must_use]
    pub fn buy_amount(mut self, amount: Amount) -> SwapBuilder<'t, SellToken, BuyToken, Set> {
        self.kind = OrderKind::Buy;
        self.amount = Some(amount);
        self.cast()
    }

    /// Sets an explicit owner. When omitted, the signer address resolves the owner at `quote`.
    #[must_use]
    pub const fn owner(mut self, owner: Address) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Sets an explicit slippage tolerance in basis points.
    #[must_use]
    pub const fn slippage_bps(mut self, slippage_bps: u32) -> Self {
        self.slippage_bps = Some(slippage_bps);
        self
    }

    /// Sets an explicit receiver.
    #[must_use]
    pub const fn receiver(mut self, receiver: Address) -> Self {
        self.receiver = Some(receiver);
        self
    }

    /// Sets a relative validity window in seconds.
    #[must_use]
    pub const fn valid_for(mut self, valid_for: u32) -> Self {
        self.valid_for = Some(valid_for);
        self
    }

    /// Sets an absolute expiry timestamp.
    #[must_use]
    pub const fn valid_to(mut self, valid_to: u32) -> Self {
        self.valid_to = Some(valid_to);
        self
    }

    /// Allows partial fills.
    #[must_use]
    pub const fn partially_fillable(mut self, partially_fillable: bool) -> Self {
        self.partially_fillable = partially_fillable;
        self
    }

    /// Attaches advanced settings (app-data, quote-request overrides, signing scheme).
    #[must_use]
    #[allow(
        clippy::missing_const_for_fn,
        reason = "overwriting the Option<TradeAdvancedSettings> runs the Arc destructor, which is not const-evaluable (E0493)"
    )]
    pub fn advanced(mut self, advanced: TradeAdvancedSettings) -> Self {
        self.advanced = Some(advanced);
        self
    }
}

impl<'t> SwapBuilder<'t, Set, Set, Set> {
    #[allow(
        clippy::missing_const_for_fn,
        reason = "the intermediate TradeParameters carries Option<AddressPerChain> whose destructor runs on each `params = params.with_*()` rebind, which is not const-evaluable (E0493)"
    )]
    fn to_trade_parameters(&self) -> TradeParameters {
        // The three required fields are guaranteed `Some` by the `<Set, Set, Set>` typestate.
        let mut params = TradeParameters::new(
            self.kind,
            self.sell_token.expect("sell_token typestate is Set"),
            self.buy_token.expect("buy_token typestate is Set"),
            self.amount.expect("amount typestate is Set"),
        )
        .with_partially_fillable(self.partially_fillable);
        if let Some(owner) = self.owner {
            params = params.with_owner(owner);
        }
        if let Some(slippage_bps) = self.slippage_bps {
            params = params.with_slippage_bps(slippage_bps);
        }
        if let Some(receiver) = self.receiver {
            params = params.with_receiver(receiver);
        }
        if let Some(valid_for) = self.valid_for {
            params = params.with_valid_for(valid_for);
        }
        if let Some(valid_to) = self.valid_to {
            params = params.with_valid_to(valid_to);
        }
        params
    }

    /// Fetches the quote for the assembled swap.
    ///
    /// The signer resolves the owner when no explicit [`SwapBuilder::owner`] was set,
    /// matching [`Trading::quote_results`]. The returned [`QuotedSwap`] can be inspected
    /// and then submitted.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when quoting fails.
    pub async fn quote<S>(self, signer: &S) -> Result<QuotedSwap<'t>, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + SignerError,
    {
        let params = self.to_trade_parameters();
        let advanced = self.advanced;
        let quote = self
            .trading
            .quote_results(params, signer, advanced.as_ref())
            .await?;
        Ok(QuotedSwap {
            trading: self.trading,
            quote,
            advanced,
        })
    }
}

/// A fetched swap quote, ready to inspect and submit. Produced by [`SwapBuilder::quote`].
#[derive(Debug)]
pub struct QuotedSwap<'t> {
    trading: &'t Trading,
    quote: QuoteResults,
    advanced: Option<TradeAdvancedSettings>,
}

impl QuotedSwap<'_> {
    /// Borrows the quote results for inspection (amounts, costs, suggested slippage) before submission.
    #[must_use]
    pub const fn results(&self) -> &QuoteResults {
        &self.quote
    }

    /// Signs and submits the exact quoted order.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when signing, app-data upload, or submission fails.
    pub async fn submit<S>(self, signer: &S) -> Result<OrderPostingResult, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + SignerError,
    {
        self.trading
            .post_swap_order_from_quote(&self.quote, signer, self.advanced.as_ref())
            .await
    }
}
