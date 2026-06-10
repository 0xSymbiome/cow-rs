#[cfg(all(
    not(feature = "browser-wallet"),
    not(feature = "in-memory-cache"),
    not(feature = "subgraph")
))]
#[test]
fn public_api_default_features_only_snapshot_matches() {
    use cow_sdk::contracts::Signature;
    use cow_sdk::core::{Address, Amount, AppCode, OrderUid, SupportedChainId};
    use cow_sdk::http::HttpTransport;
    use cow_sdk::orderbook::OrderbookApi;
    use cow_sdk::trading::{TradeParams, TraderParams, Trading, TradingBuilder, TradingOptions};
    use cow_sdk::{CowError, ErrorClass};

    let _ = core::any::type_name::<Address>();
    let _ = core::any::type_name::<Amount>();
    let _ = core::any::type_name::<AppCode>();
    let _ = core::any::type_name::<ErrorClass>();
    let _ = core::any::type_name::<dyn HttpTransport>();
    let _ = core::any::type_name::<OrderbookApi>();
    let _ = core::any::type_name::<OrderUid>();
    let _ = core::any::type_name::<CowError>();
    let _ = core::any::type_name::<Signature>();
    let _ = core::any::type_name::<SupportedChainId>();
    let _ = core::any::type_name::<TradeParams>();
    let _ = core::any::type_name::<TraderParams>();
    let _ = core::any::type_name::<Trading>();
    let _ = core::any::type_name::<TradingBuilder>();
    let _ = core::any::type_name::<TradingOptions>();

    assert_eq!(
        include_str!("fixtures/public_api_default_features_only.snap"),
        default_snapshot(),
    );
}

#[cfg(all(
    not(feature = "browser-wallet"),
    not(feature = "in-memory-cache"),
    not(feature = "subgraph")
))]
const fn default_snapshot() -> &'static str {
    "\
cow-sdk public API snapshot: default features
modules:
- app_data
- contracts
- core
- orderbook
- signing
- trading
root exports:
- ErrorClass
- RegistryError
- CowError
feature-gated exports absent:
- browser_wallet
- subgraph
"
}

#[cfg(feature = "browser-wallet")]
#[test]
fn public_api_default_features_only_snapshot_is_feature_scoped() {}
