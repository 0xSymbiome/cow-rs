use cow_sdk_orderbook::{
    ChainIdSet as OrderbookChainIdSet, ChainIdUnset as OrderbookChainIdUnset,
    EnvSet as OrderbookEnvSet, EnvUnset as OrderbookEnvUnset,
    TransportSet as OrderbookTransportSet, TransportUnset as OrderbookTransportUnset,
};
use cow_sdk_subgraph::{
    ApiKeySet as SubgraphApiKeySet, ApiKeyUnset as SubgraphApiKeyUnset,
    ChainIdSet as SubgraphChainIdSet, ChainIdUnset as SubgraphChainIdUnset,
    TransportSet as SubgraphTransportSet, TransportUnset as SubgraphTransportUnset,
};

fn main() {
    let _ = OrderbookChainIdUnset(());
    let _ = OrderbookChainIdSet(());
    let _ = OrderbookEnvUnset(());
    let _ = OrderbookEnvSet(());
    let _ = OrderbookTransportUnset(());
    let _ = OrderbookTransportSet(());

    let _ = SubgraphChainIdUnset(());
    let _ = SubgraphChainIdSet(());
    let _ = SubgraphApiKeyUnset(());
    let _ = SubgraphApiKeySet(());
    let _ = SubgraphTransportUnset(());
    let _ = SubgraphTransportSet(());
}
