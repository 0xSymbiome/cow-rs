//! Contract for the coarse-grained [`ErrorClass`] partition.
//!
//! Every public error type exposes a `class()` accessor and the facade
//! [`CowError::class()`] delegates to them, so the classification is identical
//! whether a caller holds the facade error or a bare leaf error. A throttled
//! orderbook response is retried with `Retry-After` honoring by the transport
//! layer, so a 429 that survives is an exhausted-retry throttle and reports as
//! [`ErrorClass::RateLimited`] on both the recognised-rejection (`Rejected`)
//! and unparsed-envelope (`Api`) paths.

use cow_sdk::{
    CowError, ErrorClass,
    core::{CoreError, TransportErrorClass, ValidationError},
    orderbook::{OrderbookApiError, OrderbookError, ResponseBody},
    signing::SigningError,
    trading::TradingError,
};
use serde_json::json;
use std::time::Duration;

/// Builds the promoted `Rejected` error for a recognised rejection envelope at
/// the given HTTP status (the `From` path derives the `StatusCode` from the
/// `u16`).
fn rejected(status: u16, error_type: &str) -> OrderbookError {
    OrderbookApiError::new(
        status,
        "diagnostic",
        ResponseBody::Json(json!({ "errorType": error_type, "description": "x" })),
    )
    .into()
}

/// Builds the untyped `Api` envelope error for a body with no recognisable
/// rejection envelope at the given HTTP status.
fn api(status: u16) -> OrderbookError {
    OrderbookError::Api(Box::new(OrderbookApiError::new(
        status,
        "diagnostic",
        ResponseBody::Text("opaque body".to_owned()),
    )))
}

#[test]
fn exhausted_retry_429_classifies_as_rate_limited() {
    // A 429 whose body carries no recognisable rejection envelope lands in `Api`.
    assert_eq!(
        CowError::Orderbook(api(429)).class(),
        ErrorClass::RateLimited,
    );

    // A 429 with a recognised rejection envelope lands in `Rejected`.
    assert_eq!(
        CowError::Orderbook(rejected(429, "Forbidden")).class(),
        ErrorClass::RateLimited,
    );
}

#[test]
fn non_429_remote_responses_stay_remote() {
    assert_eq!(CowError::Orderbook(api(500)).class(), ErrorClass::Remote);
    assert_eq!(
        CowError::Orderbook(rejected(400, "ZeroAmount")).class(),
        ErrorClass::Remote,
    );
}

#[test]
fn error_class_partitions_every_bucket() {
    // Validation — caller-side input boundaries.
    assert_eq!(
        CoreError::Validation(ValidationError::EmptyField { field: "appCode" }).class(),
        ErrorClass::Validation
    );
    assert_eq!(TradingError::MissingOwner.class(), ErrorClass::Validation);

    // Transport — failure before a complete response.
    assert_eq!(
        OrderbookError::Transport {
            class: TransportErrorClass::Connect,
            detail: "connect failed".to_owned().into(),
        }
        .class(),
        ErrorClass::Transport
    );

    // Remote vs RateLimited on the leaf accessor directly.
    assert_eq!(api(500).class(), ErrorClass::Remote);
    assert_eq!(api(429).class(), ErrorClass::RateLimited);

    // Signing — surfaced from the signing edge.
    assert_eq!(
        SigningError::Signer {
            operation: "typed-data signature",
            message: "signer failed".to_owned().into(),
        }
        .class(),
        ErrorClass::Signing
    );

    // Cancelled — cooperative cancellation across every error type.
    assert_eq!(CoreError::Cancelled.class(), ErrorClass::Cancelled);
    assert_eq!(OrderbookError::Cancelled.class(), ErrorClass::Cancelled);
    assert_eq!(TradingError::Cancelled.class(), ErrorClass::Cancelled);

    // Internal — invariant or helper-contract violations.
    assert_eq!(
        CoreError::Serialization("decode failed".to_owned().into()).class(),
        ErrorClass::Internal
    );
}

#[test]
fn error_class_delegates_through_trading_and_facade() {
    // The composite TradingError delegates to the wrapped error, so a wrapped
    // 429 stays RateLimited rather than collapsing to a coarse bucket.
    assert_eq!(
        TradingError::Orderbook(api(429)).class(),
        ErrorClass::RateLimited
    );

    // CowError delegates to each leaf accessor: the class is identical whether
    // a caller holds the facade error or the bare leaf error.
    let transport = OrderbookError::Transport {
        class: TransportErrorClass::Connect,
        detail: "connect failed".to_owned().into(),
    };
    let leaf_class = transport.class();
    assert_eq!(CowError::from(transport).class(), leaf_class);
    assert_eq!(
        CowError::from(TradingError::Cancelled).class(),
        ErrorClass::Cancelled
    );
}

/// Builds an untyped `Api` envelope error carrying an optional parsed
/// `Retry-After` backoff hint.
fn api_retry_after(status: u16, retry_after: Option<Duration>) -> OrderbookError {
    OrderbookError::Api(Box::new(
        OrderbookApiError::new(
            status,
            "diagnostic",
            ResponseBody::Text("opaque".to_owned()),
        )
        .with_retry_after(retry_after),
    ))
}

#[test]
fn is_retryable_delegates_through_trading_and_facade() {
    // Orderbook leaf: a server-fault status is retryable, a client-fault status
    // is not, and a transient transport class is retryable.
    assert!(api(503).is_retryable());
    assert!(!api(400).is_retryable());
    assert!(
        OrderbookError::Transport {
            class: TransportErrorClass::Timeout,
            detail: "timed out".to_owned().into(),
        }
        .is_retryable()
    );

    // The facade and the composite trading error delegate to the leaf verdict.
    assert!(CowError::Orderbook(api(503)).is_retryable());
    assert!(CowError::Trading(TradingError::Orderbook(api(503))).is_retryable());
    assert!(TradingError::Orderbook(api(429)).is_retryable());

    // Non-orderbook faults are never retryable through any layer.
    assert!(!CowError::from(TradingError::MissingOwner).is_retryable());
    assert!(!TradingError::Cancelled.is_retryable());
    assert!(!CowError::Orderbook(OrderbookError::Cancelled).is_retryable());
}

#[test]
fn backoff_hint_delegates_through_trading_and_facade() {
    // A parsed `Retry-After` surfaces through the leaf, the composite trading
    // error, and the facade unchanged.
    assert_eq!(
        api_retry_after(429, Some(Duration::from_secs(30))).backoff_hint(),
        Some(Duration::from_secs(30))
    );
    assert_eq!(
        CowError::Orderbook(api_retry_after(429, Some(Duration::from_secs(30)))).backoff_hint(),
        Some(Duration::from_secs(30))
    );
    assert_eq!(
        TradingError::Orderbook(api_retry_after(503, Some(Duration::from_secs(5)))).backoff_hint(),
        Some(Duration::from_secs(5))
    );

    // No `Retry-After` header, and non-orderbook faults, carry no hint.
    assert_eq!(
        CowError::Orderbook(api_retry_after(503, None)).backoff_hint(),
        None
    );
    assert_eq!(
        CowError::from(TradingError::MissingOwner).backoff_hint(),
        None
    );
}

/// The read-only subgraph surface joins the shared classification family when
/// the `subgraph` feature lifts it into the facade.
#[cfg(feature = "subgraph")]
mod subgraph {
    use super::{CowError, ErrorClass};
    use cow_sdk::core::TransportErrorClass;
    use cow_sdk::subgraph::{SubgraphError, SubgraphRequestErrorContext};

    /// Minimal request context for the context-carrying variants.
    fn ctx() -> Box<SubgraphRequestErrorContext> {
        Box::new(SubgraphRequestErrorContext::new(
            1,
            "https://gateway.thegraph.com/api/<redacted>/subgraphs/id/x",
            "query Totals { totals { orders } }",
            Some("Totals".to_owned()),
            None,
        ))
    }

    #[test]
    fn subgraph_error_class_partitions_every_bucket() {
        // Caller-side unsupported-chain selection is validation.
        assert_eq!(
            SubgraphError::UnsupportedNetwork { chain_id: 999_999 }.class(),
            ErrorClass::Validation
        );
        // A throttled 429 that outlived the retry budget is rate-limited; other
        // non-success statuses and GraphQL error payloads are remote.
        assert_eq!(
            SubgraphError::HttpStatus {
                context: ctx(),
                status: 429,
                body: "throttled".to_owned().into()
            }
            .class(),
            ErrorClass::RateLimited
        );
        assert_eq!(
            SubgraphError::HttpStatus {
                context: ctx(),
                status: 500,
                body: "boom".to_owned().into()
            }
            .class(),
            ErrorClass::Remote
        );
        assert_eq!(
            SubgraphError::GraphQl {
                context: ctx(),
                errors: Vec::new()
            }
            .class(),
            ErrorClass::Remote
        );
        // Transport failures stay transport; cancellation stays cancelled.
        assert_eq!(
            SubgraphError::Transport {
                context: ctx(),
                class: TransportErrorClass::Connect,
                details: "connect failed".to_owned().into(),
            }
            .class(),
            ErrorClass::Transport
        );
        assert_eq!(SubgraphError::Cancelled.class(), ErrorClass::Cancelled);
        // Empty totals and missing data are internal contract faults.
        assert_eq!(SubgraphError::NoTotalsFound.class(), ErrorClass::Internal);
        assert_eq!(
            SubgraphError::MissingData { context: ctx() }.class(),
            ErrorClass::Internal
        );
    }

    #[test]
    fn subgraph_error_class_delegates_through_facade() {
        // The facade resolves to the same class as the bare leaf error.
        let leaf = SubgraphError::UnsupportedNetwork { chain_id: 999_999 };
        let leaf_class = leaf.class();
        assert_eq!(CowError::Subgraph(leaf).class(), leaf_class);
        assert_eq!(
            CowError::from(SubgraphError::Cancelled).class(),
            ErrorClass::Cancelled
        );
    }
}
