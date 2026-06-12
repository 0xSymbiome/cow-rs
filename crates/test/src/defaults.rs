//! Panic-free canned default values used by the doubles.
//!
//! They are exposed publicly so a consumer can derive their own parameters from
//! the same values the doubles return, so a hand-built quote and the mock quote
//! never drift. Every constructor here uses an infallible path (`ZERO`,
//! `from_bytes`, `From<u64>`, `const fn new`) — there is no `unwrap`/`expect`,
//! per ADR 0033.

use cow_sdk_core::{Address, Amount, AppDataHash, Hash32, OrderUid, TransactionHash};
use cow_sdk_orderbook::{Order, OrderKind, OrderQuoteResponse, QuoteData};

/// A `valid_to` far in the future (year 2100) so canned quotes and orders never
/// read as expired.
pub const FAR_FUTURE_VALID_TO: u32 = 4_102_444_800;

/// The address the default signer reports — `0x7e5f…5bdf`, the account of the
/// secp256k1 scalar `1`.
///
/// The default [`MockSigner`](crate::MockSigner) signs with that development key
/// (the canonical key in Alloy's `signer-local` tests and the `CoW` services
/// signature-recovery vectors, never a secret), so its signatures recover to
/// this address and clear the SDK's owner-recovery gate.
#[must_use]
pub const fn address() -> Address {
    Address::from_bytes([
        0x7e, 0x5f, 0x45, 0x52, 0x09, 0x1a, 0x69, 0x12, 0x5d, 0x5d, 0xfc, 0xb7, 0xb8, 0xc2, 0x65,
        0x90, 0x29, 0x39, 0x5b, 0xdf,
    ])
}

/// The canned order UID `send_order` returns by default.
#[must_use]
pub const fn order_uid() -> OrderUid {
    OrderUid::from_bytes([0x11; 56])
}

/// The canned transaction hash `send_transaction` returns by default.
#[must_use]
pub const fn transaction_hash() -> TransactionHash {
    Hash32::from_bytes([0x33; 32])
}

/// A canned 65-byte ECDSA-shaped message signature (recovery byte `0x1b`).
#[must_use]
pub fn message_signature() -> String {
    ecdsa_shaped(0x11, 0x1b)
}

/// A canned 65-byte ECDSA-shaped typed-data signature (recovery byte `0x1c`).
#[must_use]
pub fn typed_data_signature() -> String {
    ecdsa_shaped(0x22, 0x1c)
}

/// A canned 65-byte ECDSA-shaped transaction signature (recovery byte `0x1b`).
#[must_use]
pub fn transaction_signature() -> String {
    ecdsa_shaped(0x33, 0x1b)
}

/// The canned quote `quote` returns by default: one unit sold for two,
/// valid into 2100, verified.
#[must_use]
pub fn quote() -> OrderQuoteResponse {
    OrderQuoteResponse::new(
        QuoteData::new(
            Address::ZERO,
            Address::ZERO,
            Amount::from(1_000_000_000_000_000_000_u64),
            Amount::from(2_000_000_000_000_000_000_u64),
            FAR_FUTURE_VALID_TO,
            AppDataHash::ZERO,
            OrderKind::Sell,
        ),
        "2099-01-01T00:00:00Z",
        true,
    )
    .with_id(1)
}

/// A canned open order keyed by [`order_uid`], owned by [`address`].
#[must_use]
pub fn order() -> Order {
    Order::new(
        Address::ZERO,
        Address::ZERO,
        Amount::from(1_000_000_000_000_000_000_u64),
        Amount::from(2_000_000_000_000_000_000_u64),
        FAR_FUTURE_VALID_TO,
        AppDataHash::ZERO,
        OrderKind::Sell,
        typed_data_signature(),
        Address::ZERO,
        address(),
        order_uid(),
    )
}

/// Builds a 65-byte ECDSA-shaped hex signature: 64 repeated `fill` bytes plus a
/// `recovery` byte (`0x1b`/`0x1c` are the canonical Solidity recovery ids).
fn ecdsa_shaped(fill: u8, recovery: u8) -> String {
    format!("0x{}{recovery:02x}", format!("{fill:02x}").repeat(64))
}
