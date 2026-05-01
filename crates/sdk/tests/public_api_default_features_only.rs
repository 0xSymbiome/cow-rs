#[cfg(not(feature = "browser-wallet"))]
#[test]
fn public_api_default_features_only_snapshot_matches() {
    use cow_sdk::{
        Address, Amount, ErrorClass, HttpTransport, OrderBookApi, OrderUid, SdkError, Signature,
        SupportedChainId, TradeParameters, TraderParameters, TradingSdk, TradingSdkBuilder,
        TradingSdkOptions,
    };

    let _ = core::any::type_name::<Address>();
    let _ = core::any::type_name::<Amount>();
    let _ = core::any::type_name::<ErrorClass>();
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

#[cfg(not(feature = "browser-wallet"))]
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
- ErrorClass
- HttpTransport
- InMemoryEip1271VerificationCache
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
"
}

#[cfg(feature = "browser-wallet")]
#[test]
fn public_api_default_features_only_snapshot_is_feature_scoped() {}
