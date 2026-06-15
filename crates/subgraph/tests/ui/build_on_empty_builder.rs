//! Compile-fail witness: `SubgraphApiBuilder::build` is unreachable on a fresh
//! builder with neither the chain id nor the API-key marker set; the pinned
//! `.stderr` records the diagnostic.

use cow_sdk_subgraph::SubgraphApi;

fn main() {
    let _ = SubgraphApi::builder().build();
}
