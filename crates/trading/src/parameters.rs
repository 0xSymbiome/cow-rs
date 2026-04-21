//! Offline-helper validation entry points on the public trade-parameter
//! builders.
//!
//! The builder-level subset covers the invariants a caller can check
//! without constructing a full `OrderCreation`: distinct sell / buy tokens,
//! non-zero sell / buy amounts, and a non-sentinel sell token. The full
//! reviewed protocol-invariant matrix stays on
//! [`crate::validation::OrderBoundsValidator::validate`], which is the
//! mandatory pre-transport step on every submission seam.

use cow_sdk_core::{Address, Amount, EVM_NATIVE_CURRENCY_ADDRESS};

use crate::{
    LimitTradeParameters, TradeParameters,
    validation::{AmountSide, ClientRejection},
};

fn native_sentinel() -> Address {
    Address::new(EVM_NATIVE_CURRENCY_ADDRESS)
        .expect("EVM_NATIVE_CURRENCY_ADDRESS must remain a valid address literal")
}

fn validate_distinct_tokens(
    sell_token: &Address,
    buy_token: &Address,
) -> Result<(), ClientRejection> {
    if sell_token == buy_token {
        return Err(ClientRejection::SameBuyAndSellToken {
            token: sell_token.clone(),
        });
    }
    Ok(())
}

fn validate_non_native_sell_token(sell_token: &Address) -> Result<(), ClientRejection> {
    if sell_token == &native_sentinel() {
        return Err(ClientRejection::InvalidNativeSellToken);
    }
    Ok(())
}

fn validate_non_zero_amount(side: AmountSide, amount: &Amount) -> Result<(), ClientRejection> {
    if amount.is_zero() {
        return Err(ClientRejection::ZeroAmount { side });
    }
    Ok(())
}

impl TradeParameters {
    /// Validates the builder-level subset of the reviewed protocol-invariant
    /// matrix that can be enforced without a full `OrderCreation`.
    ///
    /// Checked invariants:
    ///
    /// * distinct sell and buy tokens
    /// * non-zero sell amount
    /// * non-sentinel sell token (the native-currency sentinel belongs on
    ///   the eth-flow submission path)
    ///
    /// # Errors
    ///
    /// Returns [`ClientRejection`] on the first invariant violation.
    pub fn validate(&self) -> Result<(), ClientRejection> {
        validate_non_native_sell_token(&self.sell_token)?;
        validate_distinct_tokens(&self.sell_token, &self.buy_token)?;
        validate_non_zero_amount(AmountSide::Sell, &self.amount)?;
        Ok(())
    }
}

impl LimitTradeParameters {
    /// Validates the builder-level subset of the reviewed protocol-invariant
    /// matrix that can be enforced without a full `OrderCreation`.
    ///
    /// Checked invariants:
    ///
    /// * distinct sell and buy tokens
    /// * non-zero sell and buy amounts
    /// * non-sentinel sell token (the native-currency sentinel belongs on
    ///   the eth-flow submission path)
    ///
    /// # Errors
    ///
    /// Returns [`ClientRejection`] on the first invariant violation.
    pub fn validate(&self) -> Result<(), ClientRejection> {
        validate_non_native_sell_token(&self.sell_token)?;
        validate_distinct_tokens(&self.sell_token, &self.buy_token)?;
        validate_non_zero_amount(AmountSide::Sell, &self.sell_amount)?;
        validate_non_zero_amount(AmountSide::Buy, &self.buy_amount)?;
        Ok(())
    }
}
