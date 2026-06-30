//! Fluent, typed limit-order lifecycle on [`Trading`].
//!
//! [`Trading::limit`] opens a [`LimitBuilder`] whose sell token, buy token, sell amount,
//! and buy amount are tracked in the type system: each has its own named setter, so the
//! sell and buy sides cannot be transposed, and the terminals
//! ([`LimitBuilder::post`], [`LimitBuilder::post_presign`]) are only callable once all
//! four are supplied. A limit order carries an explicit price — the amounts you set — so
//! no quote is fetched; that is the difference from [`Trading::swap`], which quotes the
//! counter-amount. The builder mirrors the swap builder so the two read alike.

use std::marker::PhantomData;
use std::sync::Arc;

use cow_sdk_core::{Address, Amount, OrderKind, Signer, UserRejection};
use cow_sdk_signing::eip1271::Eip1271Signer;

use super::Trading;
use super::swap::{Set, Unset};
use crate::{
    Authorization, LimitTradeParams, OrderPlacement, OrderPostingResult, TradeAdvancedSettings,
    TradingError,
};

impl Trading {
    /// Opens the fluent, typed limit-order lifecycle.
    ///
    /// Supply the sell token, buy token, sell amount, and buy amount; only then do the
    /// [`LimitBuilder::post`] and [`LimitBuilder::post_presign`] terminals become
    /// available. The limit price is the amounts you supply — no quote is fetched. The
    /// order kind defaults to [`OrderKind::Sell`] (sell exactly the sell amount, receive
    /// at least the buy amount); call [`LimitBuilder::kind`] for a buy limit.
    ///
    /// Native-currency (`EthFlow`) sells go through [`Trading::swap`] instead: set the
    /// native-currency sell token and the swap path auto-routes to the on-chain `EthFlow`
    /// transaction. `limit()` is for ERC-20 limit orders and rejects a native-currency
    /// sell token.
    ///
    /// ```no_run
    /// # use cow_sdk_trading::Trading;
    /// # use cow_sdk_core::{Address, Amount, Signer, UserRejection};
    /// # async fn demo<S>(trading: &Trading, signer: &S) -> Result<(), Box<dyn std::error::Error>>
    /// # where S: Signer, S::Error: std::fmt::Display + UserRejection {
    /// let usdc = Address::ZERO;
    /// let dai = Address::ZERO;
    ///
    /// // Sell exactly 100 USDC, want at least 99 DAI, good for an hour.
    /// let posted = trading
    ///     .limit()
    ///     .sell_token(usdc)
    ///     .buy_token(dai)
    ///     .sell_amount(Amount::from_units(100, 6)?)
    ///     .buy_amount(Amount::from_units(99, 18)?)
    ///     .valid_for(3600)
    ///     .post(signer)
    ///     .await?;
    /// # let _ = posted;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub const fn limit(&self) -> LimitBuilder<'_> {
        LimitBuilder {
            trading: self,
            kind: OrderKind::Sell,
            sell_token: None,
            buy_token: None,
            sell_amount: None,
            buy_amount: None,
            owner: None,
            receiver: None,
            valid_for: None,
            valid_to: None,
            partially_fillable: false,
            quote_id: None,
            advanced: None,
            _state: PhantomData,
        }
    }
}

/// Fluent, typed builder for a limit order, reached through [`Trading::limit`].
///
/// The four type parameters track whether the sell token, buy token, sell amount, and
/// buy amount have been supplied. The [`LimitBuilder::post`] and
/// [`LimitBuilder::post_presign`] terminals exist only on
/// `LimitBuilder<Set, Set, Set, Set>`.
#[derive(Debug, Clone)]
pub struct LimitBuilder<'a, SellToken = Unset, BuyToken = Unset, SellAmt = Unset, BuyAmt = Unset> {
    trading: &'a Trading,
    kind: OrderKind,
    sell_token: Option<Address>,
    buy_token: Option<Address>,
    sell_amount: Option<Amount>,
    buy_amount: Option<Amount>,
    owner: Option<Address>,
    receiver: Option<Address>,
    valid_for: Option<u32>,
    valid_to: Option<u32>,
    partially_fillable: bool,
    quote_id: Option<i64>,
    advanced: Option<TradeAdvancedSettings>,
    _state: PhantomData<(SellToken, BuyToken, SellAmt, BuyAmt)>,
}

impl<'a, SellToken, BuyToken, SellAmt, BuyAmt>
    LimitBuilder<'a, SellToken, BuyToken, SellAmt, BuyAmt>
{
    fn cast<NextSell, NextBuy, NextSellAmt, NextBuyAmt>(
        self,
    ) -> LimitBuilder<'a, NextSell, NextBuy, NextSellAmt, NextBuyAmt> {
        LimitBuilder {
            trading: self.trading,
            kind: self.kind,
            sell_token: self.sell_token,
            buy_token: self.buy_token,
            sell_amount: self.sell_amount,
            buy_amount: self.buy_amount,
            owner: self.owner,
            receiver: self.receiver,
            valid_for: self.valid_for,
            valid_to: self.valid_to,
            partially_fillable: self.partially_fillable,
            quote_id: self.quote_id,
            advanced: self.advanced,
            _state: PhantomData,
        }
    }

    /// Sets the sell-token address.
    ///
    /// The setter is named, so the sell and buy tokens cannot be transposed the
    /// way two positional address arguments could be.
    #[must_use]
    pub fn sell_token(
        mut self,
        sell_token: Address,
    ) -> LimitBuilder<'a, Set, BuyToken, SellAmt, BuyAmt> {
        self.sell_token = Some(sell_token);
        self.cast()
    }

    /// Sets the buy-token address.
    ///
    /// The setter is named, so the sell and buy tokens cannot be transposed the
    /// way two positional address arguments could be.
    #[must_use]
    pub fn buy_token(
        mut self,
        buy_token: Address,
    ) -> LimitBuilder<'a, SellToken, Set, SellAmt, BuyAmt> {
        self.buy_token = Some(buy_token);
        self.cast()
    }

    /// Sets the exact sell amount.
    ///
    /// The setter is named, so the sell and buy amounts cannot be transposed — a
    /// transposition would silently invert the limit price.
    #[must_use]
    pub fn sell_amount(
        mut self,
        sell_amount: Amount,
    ) -> LimitBuilder<'a, SellToken, BuyToken, Set, BuyAmt> {
        self.sell_amount = Some(sell_amount);
        self.cast()
    }

    /// Sets the exact buy amount.
    ///
    /// The setter is named, so the sell and buy amounts cannot be transposed — a
    /// transposition would silently invert the limit price.
    #[must_use]
    pub fn buy_amount(
        mut self,
        buy_amount: Amount,
    ) -> LimitBuilder<'a, SellToken, BuyToken, SellAmt, Set> {
        self.buy_amount = Some(buy_amount);
        self.cast()
    }

    /// Sets the order kind. Defaults to [`OrderKind::Sell`].
    ///
    /// A sell limit sells exactly the sell amount and receives at least the buy amount;
    /// a buy limit buys exactly the buy amount and pays at most the sell amount.
    #[must_use]
    pub const fn kind(mut self, kind: OrderKind) -> Self {
        self.kind = kind;
        self
    }

    /// Sets an explicit quote id to associate with the order.
    #[must_use]
    pub const fn quote_id(mut self, quote_id: i64) -> Self {
        self.quote_id = Some(quote_id);
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

impl_common_order_builder_setters!(LimitBuilder<'a, SellToken, BuyToken, SellAmt, BuyAmt>);

impl LimitBuilder<'_, Set, Set, Set, Set> {
    /// Converts the fully-set builder into [`LimitTradeParams`].
    ///
    /// # Panics
    ///
    /// Never in practice: this method is only reachable from the
    /// `<Set, Set, Set, Set>` typestate, which guarantees the four required
    /// fields were assigned before the conversion runs.
    #[allow(
        clippy::missing_const_for_fn,
        reason = "the intermediate LimitTradeParams carries Option<AddressPerChain> whose destructor runs on each `with_*` rebind, which is not const-evaluable"
    )]
    fn to_limit_parameters(&self) -> LimitTradeParams {
        // SAFETY: the `<Set, Set, Set, Set>` typestate guarantees the four
        // required fields are `Some` before this conversion can be called.
        let mut params = LimitTradeParams::new(
            self.kind,
            self.sell_token.expect("sell_token typestate is Set"),
            self.buy_token.expect("buy_token typestate is Set"),
            self.sell_amount.expect("sell_amount typestate is Set"),
            self.buy_amount.expect("buy_amount typestate is Set"),
        )
        .with_partially_fillable(self.partially_fillable);
        if let Some(owner) = self.owner {
            params = params.with_owner(owner);
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
        if let Some(quote_id) = self.quote_id {
            params = params.with_quote_id(quote_id);
        }
        params
    }

    /// Signs and posts the limit order in one call.
    ///
    /// Use [`LimitBuilder::post_presign`] for the smart-account path that needs no signer.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when app-data generation, signing, or order submission
    /// fails.
    ///
    /// `post` is reachable only once the sell-token, buy-token, sell-amount, and
    /// buy-amount markers are all set; the limit-builder typestate makes an incomplete
    /// `limit()...post(...)` chain a compile error.
    pub async fn post<S>(self, signer: &S) -> Result<OrderPostingResult, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + UserRejection,
    {
        let params = self.to_limit_parameters();
        self.trading
            .post_limit_order(params, signer, self.advanced.as_ref())
            .await
    }

    /// Posts the limit order under the pre-sign scheme without consulting a signer.
    ///
    /// This is the smart-contract-owner path: the order is submitted with an empty
    /// signature and only becomes fillable once the owner sets the on-chain pre-signature
    /// flag. Because no signer participates, an explicit [`owner`](LimitBuilder::owner) is
    /// required.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::MissingSubmissionOwner`] when no explicit owner is set, and
    /// otherwise [`TradingError`] when app-data generation or submission fails.
    pub async fn post_presign(self) -> Result<OrderPostingResult, TradingError> {
        let params = self.to_limit_parameters();
        self.trading
            .post_limit_order_presign(params, self.advanced.as_ref())
            .await
    }

    /// Posts the limit order under the EIP-1271 scheme using a smart-account
    /// contract-signature provider (ADR 0073).
    ///
    /// Resolves to [`OrderPlacement::Live`]: an EIP-1271 order is valid once
    /// posted. Because the provider produces the signature, an explicit
    /// [`owner`](LimitBuilder::owner) is required.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::MissingSubmissionOwner`] when no explicit owner is
    /// set, and otherwise [`TradingError`] when app-data generation, signing, or
    /// submission fails.
    pub async fn post_eip1271(
        self,
        provider: Arc<dyn Eip1271Signer>,
    ) -> Result<OrderPlacement, TradingError> {
        let owner = self.owner.ok_or(TradingError::MissingSubmissionOwner)?;
        let params = self.to_limit_parameters();
        self.trading
            .place_limit(
                params,
                owner,
                Authorization::eip1271(provider),
                self.advanced.as_ref(),
            )
            .await
    }
}
