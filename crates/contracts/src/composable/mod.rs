//! Composable conditional-order framework bindings (`ComposableCoW` + TWAP).
//!
//! Conditional orders are not posted at trade time. A consumer pre-authorizes a
//! conditional order through the on-chain `ComposableCoW` registry, and an
//! off-chain watch tower polls the framework to discover when each discrete part
//! becomes tradeable and posts it to the orderbook. The owner of a conditional
//! order is a smart contract that authenticates through EIP-1271, so the
//! authorizing account is a Safe (with the `ExtensibleFallbackHandler`) or a
//! custom forwarder, never an externally owned account.
//!
//! This module is the offline encoding surface for that flow:
//!
//! - `registry` — the typed `ConditionalOrderParams`, the `conditional_order_id`
//!   derivation that matches the on-chain `ComposableCoW.hash`, and the `create`
//!   / `createWithContext` / `remove` call-data encoders.
//! - `twap` — the `TwapData` builder, its validated per-part `TwapStaticInput`,
//!   the gas-free `twap_create_transaction` / `twap_remove_transaction` builders,
//!   and the pure `TwapStaticInput::timing_at` classifier that reports which
//!   discrete part a TWAP's schedule makes live at a given moment.
//! - `multiplexer` — the `Multiplexer` merkle helper over conditional-order
//!   leaves for the `setRoot` batch path.
//!
//! Submission is the consumer's Safe; tracking reuses [`crate::compute_order_uid`]
//! over each reconstructed discrete order. The discovery reads and orderbook
//! posting stay with the consumer or the canonical watch tower; this module only
//! encodes and classifies — it never fetches, posts, or runs a loop.
//!
//! Bindings are authored inline as `alloy::sol!` against the upstream
//! cowprotocol/composable-cow Solidity surface, pinned by commit in
//! `parity/source-lock.yaml`.

/// Merkle multiplexer over conditional-order leaves for the `setRoot` batch path.
pub mod multiplexer;
/// `ComposableCoW` registry: conditional-order identity and authorization.
pub mod registry;
/// Time-weighted average price (TWAP) conditional orders.
pub mod twap;

pub use multiplexer::{Multiplexer, MultiplexerError, merkle_leaf, verify_merkle_proof};
pub use registry::{
    COMPOSABLE_COW, ConditionalOrderParams, EXTENSIBLE_FALLBACK_HANDLER, conditional_order_id,
    encode_create_calldata, encode_create_with_context_calldata, encode_remove_calldata,
};
pub use twap::{
    CURRENT_BLOCK_TIMESTAMP_FACTORY, TWAP_HANDLER, TwapBuilder, TwapData, TwapDurationOfPart,
    TwapStartTime, TwapStaticInput, TwapTiming, TwapValidationError, twap_create_transaction,
    twap_remove_transaction,
};
