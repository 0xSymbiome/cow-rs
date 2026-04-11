pub mod error;
pub mod events;
pub mod js;
pub mod mock;
pub mod provider;
pub mod signer;
pub mod wallet;

pub use error::{BrowserWalletError, RpcErrorPayload};
pub use events::{EventLog, WalletEvent, WalletSession};
pub use mock::{MockEip1193Transport, MockRequestRecord};
pub use provider::{Eip1193Provider, Eip1193Transport};
pub use signer::Eip1193Signer;
pub use wallet::{
    BrowserWallet, InjectedWalletDetectionOptions, InjectedWalletDiscovery,
    InjectedWalletDiscoverySource, InjectedWalletInfo,
};

pub use cow_sdk_core::{AsyncProvider, AsyncSigner};
