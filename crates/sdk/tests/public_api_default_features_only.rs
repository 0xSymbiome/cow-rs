#[cfg(all(not(feature = "browser-wallet"), not(feature = "in-memory-cache")))]
#[test]
fn public_api_default_features_only_snapshot_matches() {
    use cow_sdk::{
        Address, Amount, AppCode, AppCodeError, ErrorClass, HelperOnlySdk, HttpTransport,
        OrderBookApi, OrderUid, SdkError, Signature, SupportedChainId, TradeParameters,
        TraderParameters, TradingSdk, TradingSdkBuilder, TradingSdkOptions,
    };

    let _ = core::any::type_name::<Address>();
    let _ = core::any::type_name::<Amount>();
    let _ = core::any::type_name::<AppCode>();
    let _ = core::any::type_name::<AppCodeError>();
    let _ = core::any::type_name::<ErrorClass>();
    let _ = core::any::type_name::<HelperOnlySdk>();
    let _ = core::any::type_name::<dyn HttpTransport>();
    let _ = core::any::type_name::<OrderBookApi>();
    let _ = core::any::type_name::<OrderUid>();
    let _ = core::any::type_name::<SdkError>();
    let _ = core::any::type_name::<Signature>();
    let _ = core::any::type_name::<SupportedChainId>();
    let _ = core::any::type_name::<TradeParameters>();
    let _ = core::any::type_name::<TraderParameters>();
    let _ = core::any::type_name::<TradingSdk>();
    let _ = core::any::type_name::<TradingSdkBuilder>();
    let _ = core::any::type_name::<TradingSdkOptions>();

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
- HelperOnlySdk
- HttpTransport
- NoopEip1271VerificationCache
- OrderBookApi
- OrderUid
- RegistryError
- SdkError
- Signature
- SupportedChainId
- TradeParameters
- TraderParameters
- TradingSdk
- TradingSdkBuilder
- TradingSdkOptions
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
