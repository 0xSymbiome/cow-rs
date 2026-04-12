use cow_sdk_core::{Address, EVM_NATIVE_CURRENCY_ADDRESS};

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
/// Returns [`OrderbookError::InvalidTransform`] when fee fields cannot be
/// normalized as unsigned decimal strings.
pub fn transform_order(mut order: Order) -> Result<Order, OrderbookError> {
    order.total_fee = calculate_total_fee(
        order.executed_fee_amount.as_deref(),
        order.executed_fee.as_deref(),
    )?;

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

/// Adds the two orderbook fee components into the exposed `total_fee` value.
///
/// Missing components are treated as zero because the orderbook can expose
/// either or both fee fields depending on endpoint and order class.
///
/// # Errors
///
/// Returns [`OrderbookError::InvalidTransform`] when either input is not an
/// unsigned decimal string.
pub fn calculate_total_fee(
    executed_fee_amount: Option<&str>,
    executed_fee: Option<&str>,
) -> Result<String, OrderbookError> {
    add_decimal_strings(
        executed_fee_amount.unwrap_or("0"),
        executed_fee.unwrap_or("0"),
    )
}

/// Returns the order UID as a string slice for transport-layer interpolation.
#[must_use]
pub fn ensure_order_uid(uid: &OrderUid) -> &str {
    uid.as_str()
}

fn add_decimal_strings(left: &str, right: &str) -> Result<String, OrderbookError> {
    validate_decimal(left)?;
    validate_decimal(right)?;

    let mut carry = 0u32;
    let mut digits = Vec::new();
    let mut left_iter = left.as_bytes().iter().rev();
    let mut right_iter = right.as_bytes().iter().rev();

    loop {
        let left_digit = left_iter.next().map(|byte| u32::from(byte - b'0'));
        let right_digit = right_iter.next().map(|byte| u32::from(byte - b'0'));

        if left_digit.is_none() && right_digit.is_none() && carry == 0 {
            break;
        }

        let sum = left_digit.unwrap_or(0) + right_digit.unwrap_or(0) + carry;
        carry = sum / 10;
        digits.push(char::from(b'0' + (sum % 10) as u8));
    }

    digits.reverse();
    let value: String = digits.into_iter().collect();
    Ok(trim_leading_zeroes(&value))
}

fn validate_decimal(value: &str) -> Result<(), OrderbookError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(OrderbookError::InvalidTransform(format!(
            "expected unsigned decimal string, got `{value}`"
        )));
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

fn native_token_address() -> Address {
    Address::new(EVM_NATIVE_CURRENCY_ADDRESS)
        .expect("native token literal must remain a valid address")
}
