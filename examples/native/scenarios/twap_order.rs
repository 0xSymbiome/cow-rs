//! Composable TWAP: build a conditional order and its authorization call-data.
//!
//! A TWAP splits a trade into equal parts executed one per fixed interval. A
//! conditional order is owned by a smart-contract account that authenticates
//! through EIP-1271 — a Safe with the `ExtensibleFallbackHandler`, never an
//! externally owned account. This scenario builds the gas-free `createWithContext`
//! call-data and the conditional-order id used to track and later remove it. Pure
//! encoding — no network, no signer.
//!
//! `create.to` / `create.data` is the inner `ComposableCoW` call whose caller must
//! be the Safe: the Safe submits it through its own machinery (`execTransaction`,
//! or the Safe Transaction Service), so cow-sdk builds the call-data and the Safe
//! wrapping — plus the one-time `ExtensibleFallbackHandler` / domain-verifier setup
//! and the sell-token approval — stays with the consumer. Sent from an EOA the same
//! call-data registers an order no one can EIP-1271-sign.

use std::error::Error;

use serde_json::json;

use cow_sdk::composable::{TwapData, twap_create_transaction, twap_remove_transaction};
use cow_sdk::core::{Address, Amount, AppDataHash, Hash32};

fn main() -> Result<(), Box<dyn Error>> {
    // Sell 12 WETH for at least 30,000 USDC over 6 hourly parts, starting at the
    // authorization block, each part valid for its whole interval.
    let twap = TwapData::builder()
        .sell(
            Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2")?,
            Amount::new("12000000000000000000")?,
        )
        .buy(
            Address::new("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")?,
            Amount::new("30000000000")?,
        )
        .parts(6)
        .every(3600)
        .start_at_mining_time()
        .app_data(AppDataHash::new(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )?)
        .build()?;

    let salt = Hash32::from_bytes([0x42; 32]);

    // The conditional-order id matches the on-chain `ComposableCoW.hash`; it is
    // the key used to track and remove the order.
    let order_id = twap.order_id(salt)?;
    let static_input = twap.encode_static_input()?;

    // The Safe submits this inner call as its own transaction to authorize the order.
    let create = twap_create_transaction(&twap, salt)?;
    // Removing the order later is a second authorization transaction by id.
    let remove = twap_remove_transaction(order_id);

    let selector = |data: &cow_sdk::core::HexData| {
        let text = data.to_string();
        text.get(..10).map(ToString::to_string).unwrap_or(text)
    };

    let report = json!({
        "surface": "cow_sdk::composable::TwapData + twap_create_transaction",
        "orderId": order_id.to_string(),
        "staticInputBytes": static_input.len(),
        "createWithContext": {
            "to": create.to.to_hex_string(),
            "value": create.value.to_string(),
            "callDataPrefix": selector(&create.data),
        },
        "remove": {
            "to": remove.to.to_hex_string(),
            "callDataPrefix": selector(&remove.data),
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
