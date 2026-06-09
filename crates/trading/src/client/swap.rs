//! Fluent, typed swap lifecycle on [`Trading`].
//!
//! [`Trading::swap`] opens a [`SwapBuilder`] whose sell token, buy token, and
//! amount are tracked in the type system: each has its own named setter, so the
//! sell and buy addresses cannot be transposed, and the terminals
//! ([`SwapBuilder::execute`], [`SwapBuilder::quote`]) are only callable once all
//! three are supplied. The lifecycle terminates in a single asynchronous step so
//! the same chain serves every [`Signer`] backend — a local key, a remote
//! signer, a browser wallet, or a smart account.
//!
//! ```no_run
//! # use cow_sdk_trading::Trading;
//! # use cow_sdk_core::{Address, Amount, Signer, SignerError};
//! # async fn demo<S>(trading: &Trading, signer: &S) -> Result<(), Box<dyn std::error::Error>>
//! # where S: Signer, S::Error: std::fmt::Display + SignerError {
//! let usdc = Address::ZERO;
//! let weth = Address::ZERO;
//!
//! // One call quotes, signs, and submits.
//! let posted = trading
//!     .swap()
//!     .sell_token(usdc)
//!     .buy_token(weth)
//!     .sell_amount(Amount::from_units(100, 6)?)
//!     .execute(signer)
//!     .await?;
//!
//! // Or inspect the quote before committing to it.
//! let quoted = trading
//!     .swap()
//!     .sell_token(usdc)
//!     .buy_token(weth)
//!     .sell_amount(Amount::from_units(100, 6)?)
//!     .quote(signer)
//!     .await?;
//! let _costs = quoted.results().amounts_and_costs.clone();
//! let posted = quoted.submit(signer).await?;
//! # let _ = posted;
//! # Ok(())
//! # }
//! ```

use std::marker::PhantomData;

use cow_sdk_core::{Address, Amount, OrderKind, Signer, SignerError};

use super::Trading;
use crate::{OrderPostingResult, QuoteResults, TradeAdvancedSettings, TradeParameters, TradingError};

/// Typestate marker: a required swap field has not been supplied yet.
#[derive(Debug, Clone, Copy)]
pub struct Unset(());

/// Typestate marker: a required swap field has been supplied.
#[derive(Debug, Clone, Copy)]
pub struct Set(());

impl Trading {
    /// Opens the fluent, typed swap lifecycle.
    ///
    /// Supply the sell token, buy token, and an amount (through
    /// [`SwapBuilder::sell_amount`] or [`SwapBuilder::buy_amount`]); only then do
    /// the [`SwapBuilder::execute`] and [`SwapBuilder::quote`] terminals become
    /// available.
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

/// Fluent, typed builder for a swap order, reached through [`Trading::swap`].
///
/// The three type parameters track whether the sell token, buy token, and amount
/// have been supplied. The [`SwapBuilder::execute`] and [`SwapBuilder::quote`]
/// terminals exist only on `SwapBuilder<Set, Set, Set>`.
#[derive(Debug, Clone)]
pub struct SwapBuilder<'a, SellToken = Unset, BuyToken = Unset, AmountState = Unset> {
    trading: &'a Trading,
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

impl<'a, SellToken, BuyToken, AmountState> SwapBuilder<'a, SellToken, BuyToken, AmountState> {
    fn cast<NextSell, NextBuy, NextAmount>(self) -> SwapBuilder<'a, NextSell, NextBuy, NextAmount> {
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

    /// Sets the sell-token address.
    ///
    /// The setter is named, so the sell and buy tokens cannot be transposed the
    /// way two positional address arguments could be.
    #[must_use]
    pub fn sell_token(mut self, sell_token: Address) -> SwapBuilder<'a, Set, BuyToken, AmountState> {
        self.sell_token = Some(sell_token);
        self.cast()
    }

    /// Sets the buy-token address.
    ///
    /// The setter is named, so the sell and buy tokens cannot be transposed the
    /// way two positional address arguments could be.
    #[must_use]
    pub fn buy_token(mut self, buy_token: Address) -> SwapBuilder<'a, SellToken, Set, AmountState> {
        self.buy_token = Some(buy_token);
        self.cast()
    }

    /// Sets an exact sell amount, producing a sell order.
    #[must_use]
    pub fn sell_amount(mut self, amount: Amount) -> SwapBuilder<'a, SellToken, BuyToken, Set> {
        self.kind = OrderKind::Sell;
        self.amount = Some(amount);
        self.cast()
    }

    /// Sets an exact buy amount, producing a buy order.
    #[must_use]
    pub fn buy_amount(mut self, amount: Amount) -> SwapBuilder<'a, SellToken, BuyToken, Set> {
        self.kind = OrderKind::Buy;
        self.amount = Some(amount);
        self.cast()
    }

    /// Sets an explicit owner.
    ///
    /// When omitted, the signer address resolves the owner at the
    /// [`quote`](SwapBuilder::quote) or [`execute`](SwapBuilder::execute)
    /// terminal.
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

    /// Sets an explicit receiver address.
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
    #[allow(
        clippy::missing_const_for_fn,
        reason = "overwriting the Option<TradeAdvancedSettings> runs the Arc destructor, which is not const-evaluable"
    )]
    #[must_use]
    pub fn advanced(mut self, advanced: TradeAdvancedSettings) -> Self {
        self.advanced = Some(advanced);
        self
    }
}

impl<'a> SwapBuilder<'a, Set, Set, Set> {
    #[allow(
        clippy::missing_const_for_fn,
        reason = "the intermediate TradeParameters carries Option<AddressPerChain> whose destructor runs on each `with_*` rebind, which is not const-evaluable"
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

    /// Quotes, signs, and submits the swap in one call.
    ///
    /// Use [`SwapBuilder::quote`] instead when the quote should be inspected
    /// before submission. The owner is resolved from the signer address when no
    /// explicit [`owner`](SwapBuilder::owner) was set.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when quoting, signing, app-data upload, or order
    /// submission fails.
    ///
    /// `execute` is reachable only once the sell-token, buy-token, and amount
    /// markers are all set; the swap-builder typestate makes an incomplete
    /// `swap()...execute(...)` chain a compile error.
    pub async fn execute<S>(self, signer: &S) -> Result<OrderPostingResult, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + SignerError,
    {
        let params = self.to_trade_parameters();
        self.trading
            .post_swap_order(params, signer, self.advanced.as_ref())
            .await
    }

    /// Fetches the quote for the assembled swap, returning a [`QuotedSwap`] that
    /// can be inspected and then submitted.
    ///
    /// The owner is resolved from the signer address when no explicit
    /// [`owner`](SwapBuilder::owner) was set.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when quoting fails.
    pub async fn quote<S>(self, signer: &S) -> Result<QuotedSwap<'a>, TradingError>
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

/// A fetched swap quote, ready to inspect and submit. Produced by
/// [`SwapBuilder::quote`].
#[derive(Debug)]
pub struct QuotedSwap<'a> {
    trading: &'a Trading,
    quote: QuoteResults,
    advanced: Option<TradeAdvancedSettings>,
}

impl QuotedSwap<'_> {
    /// Borrows the quote results — amounts, costs, suggested slippage, and the
    /// order to sign — for inspection before submission.
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

#[cfg(test)]
mod tests {
    use super::{Set, Unset};

    #[test]
    fn swap_markers_are_sealed_against_external_construction() {
        // The private tuple field makes these constructible only inside this
        // module; external callers cannot write `Set(())` or `Unset(())`.
        let _ = Set(());
        let _ = Unset(());
    }
}
