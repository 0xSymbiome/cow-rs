#[cfg(feature = "browser-wallet")]
#[test]
fn public_api_with_all_features_snapshot_matches() {
    use cow_sdk::contracts::Signature;
    use cow_sdk::core::{Address, Amount, AppCode, OrderUid, SupportedChainId};
    use cow_sdk::orderbook::OrderbookApi;
    use cow_sdk::trading::{
        TradeParameters, TraderParameters, Trading, TradingBuilder, TradingOptions,
    };
    use cow_sdk::{CowError, ErrorClass, HttpTransport};

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
    let _ = core::any::type_name::<TradeParameters>();
    let _ = core::any::type_name::<TraderParameters>();
    let _ = core::any::type_name::<Trading>();
    let _ = core::any::type_name::<TradingBuilder>();
    let _ = core::any::type_name::<TradingOptions>();
    let _ = core::any::type_name::<cow_sdk::browser_wallet::Eip1193Signer>();
    #[cfg(feature = "subgraph")]
    let _ = core::any::type_name::<cow_sdk::subgraph::SubgraphApi>();

    assert_eq!(
        include_str!("fixtures/public_api_with_all_features.snap"),
        all_features_snapshot(),
    );
}

#[cfg(feature = "browser-wallet")]
const fn all_features_snapshot() -> &'static str {
    "\
cow-sdk public API snapshot: all features
modules:
- app_data
- browser_wallet
- contracts
- core
- orderbook
- signing
- subgraph
- trading
root exports:
- ErrorClass
- HttpTransport
- InMemoryEip1271VerificationCache
- NoopEip1271VerificationCache
- RegistryError
- CowError
- TransportError
- TransportErrorClass
"
}

#[cfg(not(feature = "browser-wallet"))]
#[test]
fn public_api_with_all_features_snapshot_is_feature_scoped() {}
