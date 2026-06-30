//! Honest EIP-1271 preflight and pre-sign lifecycle status helpers (ADR 0073).
//!
//! [`preflight_eip1271`] surfaces a real on-chain verdict for a contract
//! signature rather than silently reporting a contract scheme as valid; it wraps
//! the existing [`verify_eip1271_signature`](cow_sdk_contracts::verify_eip1271_signature)
//! primitive. [`presign_activation_status`] closes the pre-sign lifecycle on the
//! read side by collapsing the orderbook's `PresignaturePending` status to
//! `Open`, so a consumer can render "activating…/live" without re-deriving the
//! settlement event logic.

use cow_sdk_contracts::Eip1271VerificationRequest;
use cow_sdk_core::{Address, Hash32, HexData, Provider};
use cow_sdk_orderbook::{Order, OrderStatus};

use crate::TradingError;

/// Verifies a smart-contract-wallet EIP-1271 signature on-chain and returns the
/// real verdict.
///
/// Builds an [`Eip1271VerificationRequest`] for the owner contract, the order
/// digest, and the contract signature blob, then calls
/// [`verify_eip1271_signature`](cow_sdk_contracts::verify_eip1271_signature),
/// which performs the `isValidSignature` call and checks the `0x1626ba7e` magic
/// value. The check is not pre-interaction-aware: an order whose validity
/// depends on an app-data pre-hook may verify here differently from the
/// orderbook, which simulates the pre-interactions first.
///
/// # Errors
///
/// Returns [`TradingError::Contracts`] when the verifier has no code, the
/// provider call fails, or the response does not match the magic value.
pub async fn preflight_eip1271<P>(
    provider: &P,
    owner: Address,
    digest: Hash32,
    signature: HexData,
) -> Result<(), TradingError>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    let request = Eip1271VerificationRequest::new(owner, digest, signature);
    cow_sdk_contracts::verify_eip1271_signature(provider, &request)
        .await
        .map_err(TradingError::Contracts)
}

/// Collapses a pre-sign order's `PresignaturePending` status to `Open`,
/// returning every other status unchanged.
///
/// A pre-sign order is live the moment it is posted — it is open and fillable
/// once its `setPreSignature` event is indexed, and `PresignaturePending` only
/// marks the brief window before that. Both are live ([`OrderStatus::is_open`]),
/// so this helper maps the pending status onto `Open` to give a consumer one
/// "activating…/live" status without re-deriving the settlement event logic.
/// Terminal statuses (`Fulfilled`, `Cancelled`, `Expired`) pass through, so an
/// expired pre-sign order is not reported live.
#[must_use]
pub const fn presign_activation_status(order: &Order) -> OrderStatus {
    match order.status {
        OrderStatus::PresignaturePending => OrderStatus::Open,
        other => other,
    }
}
