//! In-memory test doubles for the `cow-rs` SDK public trait seams.
//!
//! `cow-sdk-test` lets a downstream application test its integration without a
//! live orderbook, RPC endpoint, or wallet. It provides
//! recording doubles for the public traits —
//! [`MockOrderbook`] ([`OrderbookClient`](cow_sdk_orderbook::OrderbookClient)),
//! [`MockSigner`] ([`Signer`](cow_sdk_core::Signer)), and [`MockProvider`]
//! ([`Provider`](cow_sdk_core::Provider) +
//! [`SigningProvider`](cow_sdk_core::SigningProvider)) — built only on the SDK's
//! public API, the `tokio-test` / `tower-test` pattern.
//!
//! Add it as a dev-dependency. Either depend on this crate directly, or enable
//! the `cow-sdk` facade's `testing` feature and reach it through
//! `cow_sdk::testing`.
//!
//! [`MockSigner`] signs with a public development key by default, so a signed
//! order recovers to the address it reports and clears the SDK's owner-recovery
//! gate. The doubles are native (`Send`, `Arc<Mutex<_>>`) and panic-free, with
//! no `unwrap`/`expect`/`panic`, per ADR 0033.
//!
//! # Example
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use cow_sdk_test::trading;
//! use cow_sdk_core::SupportedChainId;
//!
//! let testing = trading(SupportedChainId::Sepolia, "my-app")?;
//! // `testing.trading` is a real `Trading` wired to in-memory doubles. Drive
//! // it in your async test with `testing.signer` / `testing.provider`, then
//! // assert on what your code sent:
//! assert_eq!(testing.orderbook.recorded().sent_orders.len(), 0);
//! # Ok(())
//! # }
//! ```

pub mod defaults;
mod error;
mod orderbook;
mod provider;
mod signer;

pub use error::{MockError, OrderbookFailure, order_not_found, rate_limited, rejected};
pub use orderbook::{MockOrderbook, MockOrderbookBuilder, OrderbookCalls};
pub use provider::{MockProvider, MockProviderBuilder, ProviderCalls};
pub use signer::{MockSigner, MockSignerBuilder, SignerCalls};

use std::sync::Arc;

use cow_sdk_core::{CowEnv, SupportedChainId};
use cow_sdk_trading::{Trading, TradingError};

/// A [`Trading`] client pre-wired to in-memory doubles, plus the handles to
/// assert against after driving it.
#[derive(Debug)]
pub struct MockTrading {
    /// A real `Trading` client with [`MockTrading::orderbook`] injected.
    pub trading: Trading,
    /// The injected orderbook double (clone of the one inside `trading`);
    /// read [`MockOrderbook::recorded`] to assert what was sent.
    pub orderbook: MockOrderbook,
    /// A signer double to pass to signer-backed flows.
    pub signer: MockSigner,
    /// A provider double to pass to allowance, approval, and pre-sign flows.
    pub provider: MockProvider,
}

/// Builds a [`Trading`] wired to in-memory doubles for `chain` and `app_code`.
///
/// The orderbook double is injected with a context matching the client
/// (`chain`, production environment) so the SDK's construction-time context
/// validation passes.
///
/// # Errors
///
/// Returns [`TradingError`] if the SDK rejects `app_code` (the only failure
/// path; the doubles themselves never fail construction).
pub fn trading(chain: SupportedChainId, app_code: &str) -> Result<MockTrading, TradingError> {
    let orderbook = MockOrderbook::new(chain);
    let signer = MockSigner::new();
    let provider = MockProvider::builder().signer(signer.clone()).build();
    let trading = Trading::builder()
        .chain_id(chain)
        .env(CowEnv::Prod)
        .app_code(app_code)
        .orderbook_shared(Arc::new(orderbook.clone()))
        .build()?;
    Ok(MockTrading {
        trading,
        orderbook,
        signer,
        provider,
    })
}
