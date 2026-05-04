#![doc = r"
Curated first-touch exports for the primary `cow-sdk` facade.

An item belongs in this prelude when it is part of the common quote, sign,
post, app-data validation, transport/provider wiring, or primary error-handling
workflow. Specialized DTOs, helper functions, constants, and protocol-specific
utilities stay available through named-module re-exports such as
`cow_sdk::trading`, `cow_sdk::orderbook`, `cow_sdk::contracts`,
`cow_sdk::signing`, `cow_sdk::app_data`, and `cow_sdk::core`.
"]

pub use crate::{ErrorClass, SdkError};

// app_data
pub use cow_sdk_app_data::{AppDataParams, AppDataValidated};
// browser_wallet
#[cfg(feature = "browser-wallet")]
#[cfg_attr(docsrs, doc(cfg(feature = "browser-wallet")))]
pub use cow_sdk_browser_wallet::Eip1193Signer as BrowserWalletSigner;
// contracts
pub use cow_sdk_contracts::{ContractsError, Order, Signature};
// core
pub use cow_sdk_core::{
    Address, Amount, AsyncProvider, Cancellable, CowEnv, HttpTransport, OrderUid, SupportedChainId,
};
// orderbook
pub use cow_sdk_orderbook::{OrderBookApi, OrderBookApiBuilder, OrderbookError};
// signing
pub use cow_sdk_core::{AsyncSigner, Signer};
// trading
pub use cow_sdk_trading::{
    AppCode, AppCodeError, HelperOnlySdk, TradeParameters, TraderParameters, TradingError,
    TradingSdk, TradingSdkBuilder, TradingSdkOptions,
};
