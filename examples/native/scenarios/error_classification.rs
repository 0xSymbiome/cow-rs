//! Uniform error-classification tour and a realistic retry/abort decision.
//!
//! Beat 1 constructs one representative error for each `ErrorClass` bucket and
//! prints the class, proving the partition is total and consistent across the
//! facade `CowError` and the leaf `OrderbookError`.
//!
//! Beat 2 posts an order against a transport-mocked orderbook that rejects it,
//! then uses the *same* `class()` accessor — refined by
//! `OrderbookRejection::category()` — to decide whether to retry or abort.
//!
//! Beat 3 reports the status-precise retry verdict: `is_retryable()` separates a
//! retryable `503` from a non-retryable `400` (both `ErrorClass::Remote`), and
//! `backoff_hint()` surfaces the server's `Retry-After` cooldown when present.
//!
//! The class beats route every class through one `retry_disposition` helper, so
//! the tour and the realistic flow cannot disagree about what is retryable.

use std::error::Error;
use std::time::Duration;

use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use cow_sdk::core::{Amount, CowEnv, SupportedChainId, ValidationReason};
use cow_sdk::orderbook::{
    ApiContext, ExternalHostPolicy, OrderCreation, OrderbookApi, OrderbookApiError, OrderbookError,
    OrderbookRejection, OrderbookRejectionCategory, ResponseBody,
    SigningScheme as OrderbookSigningScheme,
};
use cow_sdk::signing::SigningError;
use cow_sdk::{CowError, ErrorClass};

use cow_sdk_examples_native::support::{COW, OWNER, WETH, sample_signature};

/// Application-level retry decision derived purely from the coarse class.
///
/// First cut only: `Transport` and `Remote` are retry *candidates*, `RateLimited`
/// backs off, everything else surfaces. The class alone cannot settle `Remote`
/// (a 4xx rejection is permanent, a 5xx outage is transient), so Beat 3 refines
/// the verdict with `is_retryable()` — the accessor the `ErrorClass` rustdoc
/// defers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Disposition {
    /// Re-dispatch may succeed; safe for a bounded application retry.
    Retry,
    /// The throttle outlived the transport retry budget; wait, do not spin.
    BackOff,
    /// A caller-side or protocol condition a retry cannot fix; surface it.
    Surface,
}

#[allow(
    clippy::match_same_arms,
    reason = "the explicit-variant arm and the `#[non_exhaustive]` catch-all share a body but are kept separate to document forward-compatible handling"
)]
const fn retry_disposition(class: ErrorClass) -> Disposition {
    match class {
        ErrorClass::Transport | ErrorClass::Remote => Disposition::Retry,
        ErrorClass::RateLimited => Disposition::BackOff,
        ErrorClass::Validation
        | ErrorClass::Signing
        | ErrorClass::Cancelled
        | ErrorClass::Internal => Disposition::Surface,
        // `ErrorClass` is `#[non_exhaustive]`: a class added in a future release
        // defaults to the safe disposition (surface to the caller) instead of a
        // blind retry, so this match stays forward-compatible.
        _ => Disposition::Surface,
    }
}

/// Beat 1 — build one representative error per `ErrorClass` and report it.
///
/// The representatives are deliberately a mix of facade `CowError` and leaf
/// `OrderbookError` values to show the accessor is uniform across both.
fn partition_tour() -> Vec<serde_json::Value> {
    // A non-rejection 4xx body classifies as `Remote`; a 429 body classifies as
    // `RateLimited`. Both arrive through the `OrderbookApiError -> OrderbookError`
    // conversion the transport layer uses in production.
    let remote: OrderbookError = OrderbookApiError::new(
        400,
        "Bad Request",
        ResponseBody::Text("bad request".to_owned()),
    )
    .into();
    let rate_limited: OrderbookError = OrderbookApiError::new(
        429,
        "Too Many Requests",
        ResponseBody::Text("slow down".to_owned()),
    )
    .into();

    // A serde failure on a response body becomes a structural `Serialization`
    // diagnostic, which classifies as `Internal`.
    let decode_error: OrderbookError = serde_json::from_str::<serde_json::Value>("{ not json")
        .expect_err("malformed JSON must fail to parse")
        .into();

    let representatives: Vec<(&str, CowError)> = vec![
        (
            "Validation",
            OrderbookError::InvalidQuoteRequest {
                field: "sellAmount",
                reason: ValidationReason::Missing,
            }
            .into(),
        ),
        (
            "Transport",
            OrderbookError::Transport {
                class: cow_sdk::http::TransportErrorClass::Timeout,
                detail: "connection timed out".to_owned().into(),
            }
            .into(),
        ),
        ("Remote", remote.into()),
        ("RateLimited", rate_limited.into()),
        (
            "Signing",
            // A wallet rejection (EIP-1193 code 4001) is the canonical signing
            // failure: the user declined the prompt, so a retry just re-prompts.
            SigningError::SignerRejection {
                label: "typed-data signature",
                code: 4001,
            }
            .into(),
        ),
        ("Cancelled", OrderbookError::Cancelled.into()),
        ("Internal", decode_error.into()),
    ];

    representatives
        .into_iter()
        .map(|(label, error)| {
            let class = error.class();
            json!({
                "representative": label,
                "class": format!("{class:?}"),
                "disposition": format!("{:?}", retry_disposition(class)),
            })
        })
        .collect()
}

/// Beat 2 — post an order, classify the failure, decide retry vs abort.
///
/// The mock returns the orderbook's universal rejection envelope with an
/// `InsufficientBalance` tag. Because a 400-class status is never retried by the
/// transport layer, the call returns immediately and the run stays deterministic.
async fn classify_a_real_rejection() -> Result<serde_json::Value, Box<dyn Error>> {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/orders"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "errorType": "InsufficientBalance",
            "description": "sell-side balance is below the required sell amount"
        })))
        .mount(&server)
        .await;

    let orderbook = OrderbookApi::builder_from_context(ApiContext::new(
        SupportedChainId::Sepolia,
        CowEnv::Prod,
    ))
    .external_host_policy(ExternalHostPolicy::Test)
    .base_url(server.uri())
    .build()?;

    let order = OrderCreation::new(
        WETH,
        COW,
        Amount::parse_units("0.1", 18)?,
        Amount::parse_units("0.25", 18)?,
        1_700_000_000,
        cow_sdk::core::OrderKind::Sell,
        OrderbookSigningScheme::Eip712,
        sample_signature(),
        OWNER,
    );

    let error = orderbook
        .send_order(&order)
        .await
        .expect_err("mock orderbook rejects this order with InsufficientBalance");

    // First decision: the coarse class. A structured non-2xx is `Remote`.
    let class = error.class();
    let disposition = retry_disposition(class);

    // Second decision: refine `Remote` with the action-oriented category. The
    // category names the consumer action (fund the wallet) without re-exposing
    // the redacted rejection message.
    let (rejection_category, action) = match &error {
        OrderbookError::Rejected { rejection, .. } => {
            let category = rejection.category();
            let action = match category {
                OrderbookRejectionCategory::InsufficientFunds => {
                    "fund or approve the sell token, then resubmit unchanged"
                }
                OrderbookRejectionCategory::InvalidOrder => "fix the parameters and rebuild",
                OrderbookRejectionCategory::Unfulfillable => "re-quote and try again later",
                OrderbookRejectionCategory::Conflict => "the order's state forbids this; stop",
                OrderbookRejectionCategory::NotFound => "the referenced quote/order is gone",
                OrderbookRejectionCategory::Authorization => "permission denied; escalate",
                OrderbookRejectionCategory::Server => "upstream fault; retry later",
                OrderbookRejectionCategory::Unknown => "unmodeled code; escalate",
                // `OrderbookRejectionCategory` is `#[non_exhaustive]`: a category
                // added later escalates for review rather than silently choosing
                // an action.
                _ => "unmodeled category; escalate",
            };
            // The typed tag is matchable, too — proof the partition refines a
            // concrete variant rather than a stringly-typed code.
            debug_assert!(matches!(rejection, OrderbookRejection::InsufficientBalance));
            (Some(format!("{category:?}")), action)
        }
        _ => (None, "no structured rejection envelope was present"),
    };

    Ok(json!({
        "call": "cow_sdk::orderbook::OrderbookApi::send_order",
        "class": format!("{class:?}"),
        "disposition": format!("{disposition:?}"),
        "rejectionCategory": rejection_category,
        "recommendedAction": action,
    }))
}

/// Beat 3 — the status-precise retry verdict and server backoff hint.
///
/// `class()` is the coarse telemetry bucket; `is_retryable()` is the retry
/// decision. They intentionally diverge: a retryable `503` and a non-retryable
/// `400` are both `ErrorClass::Remote`, but only the former is retryable. The
/// realistic consumer loop is `while attempts_left && error.is_retryable() {
/// sleep(error.backoff_hint().unwrap_or(default)); retry; }`.
fn retry_verdict_tour() -> Vec<serde_json::Value> {
    // A 503 with a server-pinned two-second cooldown.
    let transient: OrderbookError =
        OrderbookApiError::new(503, "Service Unavailable", ResponseBody::Empty)
            .with_retry_after(Some(Duration::from_secs(2)))
            .into();
    // A 429 throttle with no Retry-After header: retryable, but back off under
    // your own policy.
    let throttled: OrderbookError =
        OrderbookApiError::new(429, "Too Many Requests", ResponseBody::Empty).into();
    // A 400 permanent rejection: resubmitting it unchanged will not succeed.
    let permanent: OrderbookError =
        OrderbookApiError::new(400, "Bad Request", ResponseBody::Empty).into();

    [
        ("503 + Retry-After", transient),
        ("429", throttled),
        ("400", permanent),
    ]
    .into_iter()
    .map(|(case, error)| {
        json!({
            "case": case,
            "class": format!("{:?}", error.class()),
            "isRetryable": error.is_retryable(),
            "backoffHintMs": error.backoff_hint().map(|delay| delay.as_millis()),
        })
    })
    .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let report = json!({
        "surface": "cow_sdk::{CowError, ErrorClass}",
        "mode": "simulated-transport",
        "retryRule": "Transport/Remote are retry candidates, RateLimited backs off, the rest surface; is_retryable() settles Remote",
        "partitionTour": partition_tour(),
        "retryVerdict": retry_verdict_tour(),
        "realisticDecision": classify_a_real_rejection().await?,
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
