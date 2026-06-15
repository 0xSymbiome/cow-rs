//! Compile-fail witness: `OrderbookApiBuilder::build` is unreachable until the
//! environment marker is set. Reaching `.build()` on an `EnvUnset` builder is a
//! type error; the pinned `.stderr` records the diagnostic.

use cow_sdk_core::SupportedChainId;
use cow_sdk_orderbook::OrderbookApi;

fn main() {
    let _ = OrderbookApi::builder()
        .chain(SupportedChainId::Mainnet)
        .build();
}
