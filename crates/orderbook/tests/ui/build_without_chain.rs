//! Compile-fail witness: `OrderbookApiBuilder::build` is unreachable until the
//! chain id marker is set. Reaching `.build()` on a `ChainIdUnset` builder is a
//! type error; the pinned `.stderr` records the diagnostic.

use cow_sdk_core::CowEnv;
use cow_sdk_orderbook::OrderbookApi;

fn main() {
    let _ = OrderbookApi::builder().environment(CowEnv::Prod).build();
}
