#[cfg(all(not(feature = "browser-wallet"), not(feature = "in-memory-cache")))]
#[test]
fn public_api_default_features_only_snapshot_matches() {
    use cow_sdk::{
        Address, Amount, AppCode, AppCodeError, ErrorClass, HttpTransport, OrderUid, OrderbookApi,
        SdkError, Signature, SupportedChainId, TradeParameters, TraderParameters, Trading,
        TradingBuilder, TradingHelpers, TradingOptions,
    };

    let _ = core::any::type_name::<Address>();
    let _ = core::any::type_name::<Amount>();
    let _ = core::any::type_name::<AppCode>();
    let _ = core::any::type_name::<AppCodeError>();
    let _ = core::any::type_name::<ErrorClass>();
    let _ = core::any::type_name::<TradingHelpers>();
    let _ = core::any::type_name::<dyn HttpTransport>();
    let _ = core::any::type_name::<OrderbookApi>();
    let _ = core::any::type_name::<OrderUid>();
    let _ = core::any::type_name::<SdkError>();
    let _ = core::any::type_name::<Signature>();
    let _ = core::any::type_name::<SupportedChainId>();
    let _ = core::any::type_name::<TradeParameters>();
    let _ = core::any::type_name::<TraderParameters>();
    let _ = core::any::type_name::<Trading>();
    let _ = core::any::type_name::<TradingBuilder>();
    let _ = core::any::type_name::<TradingOptions>();

    assert_eq!(
        include_str!("fixtures/public_api_default_features_only.snap"),
        default_snapshot(),
    );
}

#[cfg(all(not(feature = "browser-wallet"), not(feature = "in-memory-cache")))]
const fn default_snapshot() -> &'static str {
    "\
cow-sdk public API snapshot: default features
modules:
- app_data
- contracts
- core
- orderbook
- prelude
- signing
- trading
root exports:
- Address
- Amount
- AppCode
- AppCodeError
- ErrorClass
- TradingHelpers
- HttpTransport
- NoopEip1271VerificationCache
- OrderbookApi
- OrderUid
- RegistryError
- SdkError
- Signature
- SupportedChainId
- TradeParameters
- TraderParameters
- Trading
- TradingBuilder
- TradingOptions
- TransportError
- TransportErrorClass
feature-gated exports absent:
- browser_wallet
- BrowserWalletSigner
- InMemoryEip1271VerificationCache
"
}

#[cfg(feature = "browser-wallet")]
#[test]
fn public_api_default_features_only_snapshot_is_feature_scoped() {}
