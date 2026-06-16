#[test]
fn headline_types_stay_reachable_with_all_features() {
    use cow_sdk::contracts::Signature;
    use cow_sdk::core::{Address, Amount, AppCode, OrderUid, SupportedChainId};
    use cow_sdk::http::HttpTransport;
    use cow_sdk::orderbook::OrderbookApi;
    use cow_sdk::trading::{TradeParams, TraderParams, Trading, TradingBuilder};
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
    #[cfg(feature = "subgraph")]
    let _ = core::any::type_name::<cow_sdk::subgraph::SubgraphApi>();
}
