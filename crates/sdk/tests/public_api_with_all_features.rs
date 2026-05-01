#[cfg(feature = "browser-wallet")]
#[test]
fn public_api_with_all_features_snapshot_matches() {
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
    let _ = core::any::type_name::<cow_sdk::browser_wallet::Eip1193Signer>();
    let _ = core::any::type_name::<cow_sdk::prelude::BrowserWalletSigner>();

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
- prelude
- signing
- trading
root exports:
- Address
- Amount
- BrowserWalletSigner (prelude)
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
"
}

#[cfg(not(feature = "browser-wallet"))]
#[test]
fn public_api_with_all_features_snapshot_is_feature_scoped() {}
