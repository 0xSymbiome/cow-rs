//! Compile-fail witness: `SubgraphApiBuilder::build` is unreachable until the
//! API-key marker is set. Reaching `.build()` on an `ApiKeyUnset` builder is a
//! type error; the pinned `.stderr` records the diagnostic.

use cow_sdk_core::SupportedChainId;
use cow_sdk_subgraph::SubgraphApi;

fn main() {
    let _ = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .build();
}
