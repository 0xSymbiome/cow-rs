//! Contract for `OrderbookRejection::category()`.
//!
//! The coarse [`OrderbookRejectionCategory`] names the consumer action a
//! rejection calls for. The accessor is additive — it keeps the full typed
//! taxonomy — and partitions every variant with no wildcard arm, so a future
//! wire tag must be assigned a category at the source rather than being
//! silently misclassified. The cases below pin one representative per category
//! plus the assignments most likely to be debated.

use cow_sdk_core::Amount;
use cow_sdk_orderbook::{OrderbookRejection, OrderbookRejectionCategory};

#[test]
fn category_maps_representative_and_borderline_variants() {
    use OrderbookRejection as R;
    use OrderbookRejectionCategory as C;

    let cases: [(R, C); 22] = [
        // Authorization — escalate; not param-fixable.
        (R::Forbidden, C::Authorization),
        // InsufficientFunds — fund/approve, resubmit unchanged.
        (R::InsufficientBalance, C::InsufficientFunds),
        (R::InsufficientAllowance, C::InsufficientFunds),
        // NotFound — missing resource (distinct from re-quote).
        (R::QuoteNotFound, C::NotFound),
        (R::OrderNotFound, C::NotFound),
        (
            R::NotFound {
                message: "x".to_owned().into(),
            },
            C::NotFound,
        ),
        // Conflict — terminal lifecycle / routing state.
        (R::DuplicatedOrder, C::Conflict),
        (R::AlreadyCancelled, C::Conflict),
        (R::OnChainOrder, C::Conflict),
        // Unfulfillable — market/liquidity/fee economics, may clear later.
        (R::NoLiquidity, C::Unfulfillable),
        (R::InsufficientLiquidity, C::Unfulfillable),
        (R::TokenTemporarilySuspended, C::Unfulfillable),
        // Fee-coverage shortfall is economic/quote-time (upstream groups it with
        // `NoLiquidity`), not a malformed request.
        (
            R::SellAmountDoesNotCoverFee {
                fee_amount: Amount::new("1").expect("fee amount fixture parses"),
            },
            C::Unfulfillable,
        ),
        // Server — server-side fault.
        (R::InternalServerError, C::Server),
        (R::MetadataSerializationFailed, C::Server),
        // InvalidOrder — fix the request and rebuild (incl. borderlines).
        (R::ZeroAmount, C::InvalidOrder),
        (R::InvalidSignature, C::InvalidOrder),
        (R::TransferSimulationFailed, C::InvalidOrder),
        (R::QuoteNotVerified, C::InvalidOrder),
        (R::InvalidNativeSellToken, C::InvalidOrder),
        (
            R::AppDataInvalid {
                message: "x".to_owned().into(),
            },
            C::InvalidOrder,
        ),
        // Unknown — forward-compatibility fallback.
        (
            R::Unknown {
                code: "NewServicesTag".into(),
                message: "x".to_owned().into(),
            },
            C::Unknown,
        ),
    ];

    for (rejection, expected) in &cases {
        assert_eq!(
            rejection.category(),
            *expected,
            "unexpected category for {rejection:?}"
        );
    }
}
