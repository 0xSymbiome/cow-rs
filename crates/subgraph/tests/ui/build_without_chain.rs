//! Compile-fail witness: `SubgraphApiBuilder::build` is unreachable until the
//! chain id marker is set. Reaching `.build()` on a `ChainIdUnset` builder is a
//! type error; the pinned `.stderr` records the diagnostic.

use cow_sdk_subgraph::SubgraphApi;

fn main() {
    let _ = SubgraphApi::builder().api_key("partner-key").build();
}
