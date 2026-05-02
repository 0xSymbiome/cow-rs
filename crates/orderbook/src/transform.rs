use cow_sdk_core::{Address, Amount, EVM_NATIVE_CURRENCY_ADDRESS};

use crate::{
    error::OrderbookError,
    types::{Order, OrderUid},
};

/// Normalizes an orderbook order response into the crate's stable DTO contract.
///
/// This updates `EthFlow` orders so the user-visible owner, validity, and native
/// token address match the effective order semantics exposed by the orderbook.
///
/// # Errors
///
/// Returns [`OrderbookError::InvalidTransform`] when the executed-fee field
/// cannot be normalized as an unsigned decimal string.
pub fn transform_order(mut order: Order) -> Result<Order, OrderbookError> {
    let executed_fee = order.executed_fee.as_ref().map(ToString::to_string);
    order.total_fee = calculate_total_fee(executed_fee.as_deref())?;

    if let Some(ethflow_data) = &order.ethflow_data {
        order.valid_to = ethflow_data.user_valid_to;
        if let Some(onchain_user) = &order.onchain_user {
            order.owner = onchain_user.clone();
        }
        order.sell_token = native_token_address();
    }

    Ok(order)
}

/// Applies [`transform_order`] to every order in the provided response list.
///
/// # Errors
///
/// Returns the first error produced while normalizing an individual order.
pub fn transform_orders(orders: Vec<Order>) -> Result<Vec<Order>, OrderbookError> {
    orders.into_iter().map(transform_order).collect()
}

/// Normalizes the executed-fee component into the exposed `total_fee` value.
///
/// A missing executed fee is treated as zero; the services schema uses
/// `executedFee` as the canonical executed-fee exposure and
/// `protocolFeeBps` for protocol-fee descriptors on the quote response.
///
/// # Errors
///
/// Returns [`OrderbookError::InvalidTransform`] when the input is not an
/// unsigned decimal string.
pub fn calculate_total_fee(executed_fee: Option<&str>) -> Result<Amount, OrderbookError> {
    let value = executed_fee.unwrap_or("0");
    validate_decimal(value)?;
    Amount::new(trim_leading_zeroes(value)).map_err(|_| OrderbookError::InvalidTransform {
        field: "executedFee",
        reason: cow_sdk_core::ValidationReason::BadShape {
            details: "expected unsigned decimal string",
        },
    })
}

/// Returns the order UID as a string slice for transport-layer interpolation.
#[must_use]
pub fn ensure_order_uid(uid: &OrderUid) -> &str {
    uid.as_str()
}

fn validate_decimal(value: &str) -> Result<(), OrderbookError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(OrderbookError::InvalidTransform {
            field: "executedFee",
            reason: cow_sdk_core::ValidationReason::BadShape {
                details: "expected unsigned decimal string",
            },
        });
    }

    Ok(())
}

fn trim_leading_zeroes(value: &str) -> String {
    let trimmed = value.trim_start_matches('0');
    if trimmed.is_empty() {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}

/// Returns the orderbook native-token sentinel address.
///
/// # Panics
///
/// Panics only if the shared native-currency sentinel literal stops being a
/// valid EVM address.
fn native_token_address() -> Address {
    // SAFETY: EVM_NATIVE_CURRENCY_ADDRESS is a crate-owned protocol sentinel
    // literal validated through the shared Address constructor.
    Address::new(EVM_NATIVE_CURRENCY_ADDRESS)
        .expect("native token literal must remain a valid address")
}
