//! Compile-fail witness: `OrderbookApiBuilder::build` is unreachable on a fresh
//! builder with neither the chain id nor the environment marker set; the pinned
//! `.stderr` records the diagnostic.

use cow_sdk_orderbook::OrderbookApi;

fn main() {
    let _ = OrderbookApi::builder().build();
}
