//! Offline-helper validation entry points on the public trade-parameter
//! builders.
//!
//! The builder-level subset covers the invariants a caller can check
//! without constructing a full `OrderCreation`: buy-side same-token rejection,
//! non-zero sell / buy amounts, and a non-sentinel sell token. The full
//! reviewed protocol-invariant matrix stays on
//! [`crate::validation::OrderBoundsValidator::validate`], which is the
//! mandatory pre-transport step on every submission seam.

use cow_sdk_app_data::{AppDataError, PartnerFee};
use cow_sdk_core::{Address, Amount, EVM_NATIVE_CURRENCY_ADDRESS, OrderKind, ValidationReason};

use crate::{
    LimitTradeParameters, TradeParameters,
    validation::{AmountSide, ClientRejection},
};

/// Returns the native-currency sentinel address used in trade-parameter checks.
///
/// # Panics
///
/// Panics only if the shared native-currency sentinel literal stops being a
/// valid EVM address.
fn native_sentinel() -> Address {
    // SAFETY: EVM_NATIVE_CURRENCY_ADDRESS is a crate-owned protocol sentinel
    // literal validated through the shared Address constructor.
    Address::new(EVM_NATIVE_CURRENCY_ADDRESS)
        .expect("EVM_NATIVE_CURRENCY_ADDRESS must remain a valid address literal")
}

fn validate_distinct_tokens(
    sell_token: &Address,
    buy_token: &Address,
    kind: OrderKind,
) -> Result<(), ClientRejection> {
    if sell_token == buy_token && kind == OrderKind::Buy {
        return Err(ClientRejection::SameBuyAndSellToken { token: *sell_token });
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

fn validate_partner_fee(partner_fee: Option<&PartnerFee>) -> Result<(), ClientRejection> {
    let Some(partner_fee) = partner_fee else {
        return Ok(());
    };

    partner_fee.validate().map_err(|error| match error {
        AppDataError::InvalidPartnerFee { field, reason } => {
            ClientRejection::InvalidPartnerFee { field, reason }
        }
        _ => ClientRejection::InvalidPartnerFee {
            field: "partnerFee",
            reason: ValidationReason::Precondition {
                details: "partner fee metadata must satisfy the app-data policy",
            },
        },
    })
}

impl TradeParameters {
    /// Validates the builder-level subset of the reviewed protocol-invariant
    /// matrix that can be enforced without a full `OrderCreation`.
    ///
    /// Checked invariants:
    ///
    /// * buy-side same-token orders
    /// * non-zero sell amount
    /// * non-sentinel sell token (the native-currency sentinel belongs on
    ///   the eth-flow submission path)
    /// * valid partner-fee recipient when a partner-fee policy is present
    ///
    /// # Errors
    ///
    /// Returns [`ClientRejection`] on the first invariant violation.
    pub fn validate(&self) -> Result<(), ClientRejection> {
        validate_non_native_sell_token(&self.sell_token)?;
        validate_distinct_tokens(&self.sell_token, &self.buy_token, self.kind)?;
        validate_non_zero_amount(AmountSide::Sell, &self.amount)?;
        validate_partner_fee(self.partner_fee.as_ref())?;
        Ok(())
    }
}

impl LimitTradeParameters {
    /// Validates the builder-level subset of the reviewed protocol-invariant
    /// matrix that can be enforced without a full `OrderCreation`.
    ///
    /// Checked invariants:
    ///
    /// * buy-side same-token orders
    /// * non-zero sell and buy amounts
    /// * non-sentinel sell token (the native-currency sentinel belongs on
    ///   the eth-flow submission path)
    /// * valid partner-fee recipient when a partner-fee policy is present
    ///
    /// # Errors
    ///
    /// Returns [`ClientRejection`] on the first invariant violation.
    pub fn validate(&self) -> Result<(), ClientRejection> {
        validate_non_native_sell_token(&self.sell_token)?;
        validate_distinct_tokens(&self.sell_token, &self.buy_token, self.kind)?;
        validate_non_zero_amount(AmountSide::Sell, &self.sell_amount)?;
        validate_non_zero_amount(AmountSide::Buy, &self.buy_amount)?;
        validate_partner_fee(self.partner_fee.as_ref())?;
        Ok(())
    }
}
