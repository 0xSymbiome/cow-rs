//! Shared retry driver for SDK HTTP clients.
//!
//! [`run_with_retry`] is the single retry loop used by the orderbook, subgraph,
//! and IPFS clients. It owns the attempt loop, rate-limit acquisition, the
//! retry decisions ([`RetryPolicy::should_retry_status`],
//! [`RetryPolicy::should_retry_network`], [`RetryPolicy::delay_for_attempt`],
//! [`RetryPolicy::delay_for_status`]), the wasm-safe `Retry-After` clock
//! ([`crate::time::system_now`]), and retry telemetry.
//!
//! Per-client behavior stays in the calling crate: the attempt closure performs
//! the dispatch and classifies the result into an [`AttemptOutcome`], building
//! the caller's own typed error for the terminal path. This keeps one retry,
//! backoff, and `Retry-After` contract while letting each client own its
//! payload type, success decoding, error type, and rate-limiter scope.
//!
//! Cancellation is cooperative and external: callers compose
//! [`cow_sdk_core::Cancellable::cancel_with`] at the call site, which drops the
//! returned future — and any in-flight rate-limit acquire or backoff sleep —
//! when the token fires. The driver installs no hidden cancellation state.

use std::future::Future;
use std::time::{Duration, SystemTime};

use cow_sdk_core::{CancellationToken, TransportErrorClass};
use url::Url;

use crate::{NetworkErrorKind, RequestRateLimiter, RetryPolicy};

/// Classification of a failed attempt, used by [`run_with_retry`] to decide
/// retryability and the backoff delay.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetrySignal {
    /// The endpoint returned an HTTP status response. Retryability follows
    /// [`RetryPolicy::should_retry_status`]; the delay follows
    /// [`RetryPolicy::delay_for_status`] with these response headers, so a
    /// `Retry-After` header on `429`/`503` is honored.
    HttpStatus {
        /// HTTP status code.
        status: u16,
        /// Response headers, read for a `Retry-After` value.
        headers: Vec<(String, String)>,
    },
    /// The request failed before producing an HTTP status. Retryability follows
    /// [`RetryPolicy::should_retry_network`]; the delay follows
    /// [`RetryPolicy::delay_for_attempt`].
    Transport {
        /// Categorical transport failure.
        class: TransportErrorClass,
    },
}

/// The result of a single attempt driven by [`run_with_retry`].
///
/// The error type `E` is the caller's own typed error. On a non-retried
/// [`AttemptOutcome::Failure`] the runner returns the carried `error` verbatim,
/// so the caller controls the terminal error shape while the runner controls
/// the retry decision.
#[non_exhaustive]
#[derive(Debug)]
pub enum AttemptOutcome<T, E> {
    /// The attempt succeeded; [`run_with_retry`] returns `Ok(value)`.
    Success(T),
    /// The attempt failed.
    Failure {
        /// Terminal error returned when this outcome is not retried.
        error: E,
        /// Retry classification driving the backoff decision.
        signal: RetrySignal,
    },
}

/// Selects which rate-limiter bucket an attempt draws a token from.
#[non_exhaustive]
#[derive(Debug)]
pub enum LimiterKey<'a> {
    /// Draw from the shared global bucket.
    Global,
    /// Draw from the per-host bucket keyed by this URL.
    PerUrl(&'a Url),
}

/// Drives an attempt closure under the supplied retry and rate-limit policy.
///
/// `attempt` runs once per attempt with the 1-based attempt index. For each
/// attempt the runner acquires a rate-limit token, runs `attempt`, and on
/// [`AttemptOutcome::Failure`] consults `policy` to either back off and retry or
/// return the terminal error. Backoff for `429`/`503` honors a `Retry-After`
/// response header through the wasm-safe [`crate::time::system_now`] clock; a
/// non-retryable signal returns immediately without re-dispatching.
///
/// No `Send` bound is imposed on the attempt future, so the same driver serves
/// native (`Send`) and browser (`?Send`) transports.
///
/// Cancellation is external: wrap the returned future with
/// [`cow_sdk_core::Cancellable::cancel_with`] to drop it (and any in-flight
/// acquire or backoff sleep) when a token fires.
///
/// # Errors
///
/// Returns the terminal `E` from the last non-retried [`AttemptOutcome::Failure`]
/// — a non-retryable signal, or a retryable signal once `policy.max_attempts()`
/// is reached.
pub async fn run_with_retry<T, E, F, Fut>(
    policy: &RetryPolicy,
    rate_limiter: &RequestRateLimiter,
    limiter_key: LimiterKey<'_>,
    attempt: F,
) -> Result<T, E>
where
    F: FnMut(usize) -> Fut,
    Fut: Future<Output = AttemptOutcome<T, E>>,
{
    run_with_retry_using(
        policy,
        rate_limiter,
        limiter_key,
        attempt,
        crate::time::sleep,
        crate::time::system_now,
    )
    .await
}

/// Implementation core of [`run_with_retry`] with the backoff sleeper and the
/// wall clock injected.
///
/// Production callers reach this through [`run_with_retry`], which supplies
/// [`crate::time::sleep`] and [`crate::time::system_now`]. Tests inject a
/// recording sleeper and a fixed clock to assert the deterministic delay
/// sequence without real time.
pub(crate) async fn run_with_retry_using<T, E, F, Fut, S, SFut, C>(
    policy: &RetryPolicy,
    rate_limiter: &RequestRateLimiter,
    limiter_key: LimiterKey<'_>,
    mut attempt: F,
    mut sleeper: S,
    clock: C,
) -> Result<T, E>
where
    F: FnMut(usize) -> Fut,
    Fut: Future<Output = AttemptOutcome<T, E>>,
    S: FnMut(Duration) -> SFut,
    SFut: Future<Output = ()>,
    C: Fn() -> SystemTime,
{
    let max_attempts = policy.max_attempts().max(1);
    let mut attempt_index = 1usize;

    loop {
        // Cancellation is external (drop-based via `cancel_with`); a throwaway
        // token satisfies the limiter API without installing hidden global
        // cancellation state. The token is never fired, so acquire cannot
        // report cancellation through this path.
        let token = CancellationToken::new();
        match &limiter_key {
            LimiterKey::Global => {
                let _ = rate_limiter.acquire_global(&token).await;
            }
            LimiterKey::PerUrl(url) => {
                let _ = rate_limiter.acquire(url, &token).await;
            }
        }

        match attempt(attempt_index).await {
            AttemptOutcome::Success(value) => return Ok(value),
            AttemptOutcome::Failure { error, signal } => {
                let retryable = match &signal {
                    RetrySignal::HttpStatus { status, .. } => policy.should_retry_status(*status),
                    RetrySignal::Transport { class } => policy
                        .should_retry_network(NetworkErrorKind::from_transport_error_class(*class)),
                };

                if retryable && attempt_index < max_attempts {
                    let delay = match &signal {
                        RetrySignal::HttpStatus { status, headers } => {
                            policy.delay_for_status(attempt_index, *status, headers, clock())
                        }
                        RetrySignal::Transport { .. } => policy.delay_for_attempt(attempt_index),
                    };
                    emit_retry_event(attempt_index, &signal, delay);
                    sleeper(delay).await;
                    attempt_index += 1;
                    continue;
                }

                if retryable {
                    // Retryable signal, but `max_attempts` has been reached.
                    emit_exhausted_event(attempt_index, &signal);
                }
                return Err(error);
            }
        }
    }
}

#[cfg(feature = "tracing")]
fn emit_retry_event(attempt_index: usize, signal: &RetrySignal, delay: Duration) {
    let attempt_index = u64::try_from(attempt_index).unwrap_or(u64::MAX);
    let backoff_ms = u64::try_from(delay.as_millis()).unwrap_or(u64::MAX);
    match signal {
        RetrySignal::HttpStatus { status, .. } => tracing::debug!(
            target: "cow_sdk::transport",
            attempt_index,
            status = u64::from(*status),
            backoff_ms,
            "retry scheduled after status response"
        ),
        RetrySignal::Transport { class } => tracing::debug!(
            target: "cow_sdk::transport",
            attempt_index,
            transport_error_class = class.as_str(),
            backoff_ms,
            "retry scheduled after transport error"
        ),
    }
}

#[cfg(not(feature = "tracing"))]
#[inline]
const fn emit_retry_event(_attempt_index: usize, _signal: &RetrySignal, _delay: Duration) {}

#[cfg(feature = "tracing")]
fn emit_exhausted_event(attempt_index: usize, signal: &RetrySignal) {
    let attempt_index = u64::try_from(attempt_index).unwrap_or(u64::MAX);
    match signal {
        RetrySignal::HttpStatus { status, .. } => tracing::warn!(
            target: "cow_sdk::transport",
            attempt_index,
            status = u64::from(*status),
            backoff_ms = 0_u64,
            "retry attempts exhausted after status response"
        ),
        RetrySignal::Transport { class } => tracing::warn!(
            target: "cow_sdk::transport",
            attempt_index,
            transport_error_class = class.as_str(),
            backoff_ms = 0_u64,
            "retry attempts exhausted after transport error"
        ),
    }
}

#[cfg(not(feature = "tracing"))]
#[inline]
const fn emit_exhausted_event(_attempt_index: usize, _signal: &RetrySignal) {}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::{Duration, SystemTime};

    use cow_sdk_core::TransportErrorClass;

    use super::{AttemptOutcome, LimiterKey, RetrySignal, run_with_retry_using};
    use crate::{JitterStrategy, RequestRateLimiter, RetryPolicy};

    const NOW_SECS: u64 = 1_000_000;

    /// Scripted raw transport result for one attempt.
    #[derive(Clone, Copy)]
    enum Raw {
        Ok,
        Status {
            status: u16,
            retry_after: RetryAfter,
        },
        Transport(TransportErrorClass),
    }

    #[derive(Clone, Copy)]
    enum RetryAfter {
        None,
        DeltaSecs(u64),
        HttpDateAtSecs(u64),
    }

    #[derive(Debug, PartialEq, Eq)]
    enum Outcome {
        Success,
        Status(u16),
        Transport(TransportErrorClass),
    }

    struct RunReport {
        outcome: Outcome,
        sleeps_ms: Vec<u64>,
        dispatches: usize,
    }

    fn fixed_clock() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(NOW_SECS)
    }

    /// Deterministic policy: documented backoff, jitter disabled so the delay
    /// sequence is exact.
    fn policy(max_attempts: usize) -> RetryPolicy {
        RetryPolicy::builder()
            .max_attempts(max_attempts)
            .jitter(JitterStrategy::none())
            .build()
    }

    fn header_pairs(retry_after: RetryAfter) -> Vec<(String, String)> {
        match retry_after {
            RetryAfter::None => Vec::new(),
            RetryAfter::DeltaSecs(secs) => vec![("retry-after".to_owned(), secs.to_string())],
            RetryAfter::HttpDateAtSecs(at) => {
                let when = SystemTime::UNIX_EPOCH + Duration::from_secs(at);
                vec![("retry-after".to_owned(), httpdate::fmt_http_date(when))]
            }
        }
    }

    async fn run(script: &[Raw], max_attempts: usize) -> RunReport {
        let sleeps: Rc<RefCell<Vec<u64>>> = Rc::new(RefCell::new(Vec::new()));
        let dispatches = Rc::new(RefCell::new(0usize));
        let policy = policy(max_attempts);
        let limiter = RequestRateLimiter::unlimited();

        let script = script.to_vec();
        let sleeps_for_sleeper = Rc::clone(&sleeps);
        let dispatches_for_attempt = Rc::clone(&dispatches);

        let outcome = run_with_retry_using::<Outcome, Outcome, _, _, _, _, _>(
            &policy,
            &limiter,
            LimiterKey::Global,
            move |attempt_index| {
                let idx = (attempt_index - 1).min(script.len() - 1);
                let raw = script[idx];
                *dispatches_for_attempt.borrow_mut() += 1;
                async move {
                    match raw {
                        Raw::Ok => AttemptOutcome::Success(Outcome::Success),
                        Raw::Status {
                            status,
                            retry_after,
                        } => AttemptOutcome::Failure {
                            error: Outcome::Status(status),
                            signal: RetrySignal::HttpStatus {
                                status,
                                headers: header_pairs(retry_after),
                            },
                        },
                        Raw::Transport(class) => AttemptOutcome::Failure {
                            error: Outcome::Transport(class),
                            signal: RetrySignal::Transport { class },
                        },
                    }
                }
            },
            move |delay: Duration| {
                sleeps_for_sleeper
                    .borrow_mut()
                    .push(u64::try_from(delay.as_millis()).unwrap_or(u64::MAX));
                async {}
            },
            fixed_clock,
        )
        .await;

        let outcome = match outcome {
            Ok(value) | Err(value) => value,
        };
        RunReport {
            outcome,
            sleeps_ms: Rc::try_unwrap(sleeps).unwrap().into_inner(),
            dispatches: *dispatches.borrow(),
        }
    }

    #[tokio::test]
    async fn immediate_success_does_not_sleep() {
        let report = run(&[Raw::Ok], 10).await;
        assert_eq!(report.outcome, Outcome::Success);
        assert_eq!(report.dispatches, 1);
        assert!(report.sleeps_ms.is_empty());
    }

    #[tokio::test]
    async fn retryable_status_then_success_backs_off_once() {
        let report = run(
            &[
                Raw::Status {
                    status: 503,
                    retry_after: RetryAfter::None,
                },
                Raw::Ok,
            ],
            10,
        )
        .await;
        assert_eq!(report.outcome, Outcome::Success);
        assert_eq!(report.dispatches, 2);
        assert_eq!(report.sleeps_ms, vec![50]);
    }

    #[tokio::test]
    async fn delta_retry_after_overrides_backoff_floor() {
        let report = run(
            &[
                Raw::Status {
                    status: 429,
                    retry_after: RetryAfter::DeltaSecs(2),
                },
                Raw::Ok,
            ],
            10,
        )
        .await;
        assert_eq!(report.outcome, Outcome::Success);
        assert_eq!(report.sleeps_ms, vec![2000]);
    }

    #[tokio::test]
    async fn http_date_retry_after_uses_the_injected_clock() {
        let report = run(
            &[
                Raw::Status {
                    status: 503,
                    retry_after: RetryAfter::HttpDateAtSecs(NOW_SECS + 10),
                },
                Raw::Ok,
            ],
            10,
        )
        .await;
        assert_eq!(report.outcome, Outcome::Success);
        // 10 seconds in the future against the fixed clock, not a multi-decade
        // delay computed against UNIX_EPOCH.
        assert_eq!(report.sleeps_ms, vec![10_000]);
    }

    #[tokio::test]
    async fn persistent_retryable_status_exhausts_attempts() {
        let report = run(
            &[Raw::Status {
                status: 500,
                retry_after: RetryAfter::None,
            }],
            10,
        )
        .await;
        assert_eq!(report.outcome, Outcome::Status(500));
        assert_eq!(report.dispatches, 10);
        assert_eq!(
            report.sleeps_ms,
            vec![50, 100, 200, 400, 800, 1600, 3200, 3200, 3200]
        );
    }

    #[tokio::test]
    async fn persistent_transport_error_exhausts_attempts() {
        let report = run(&[Raw::Transport(TransportErrorClass::Timeout)], 10).await;
        assert_eq!(
            report.outcome,
            Outcome::Transport(TransportErrorClass::Timeout)
        );
        assert_eq!(report.dispatches, 10);
        assert_eq!(
            report.sleeps_ms,
            vec![50, 100, 200, 400, 800, 1600, 3200, 3200, 3200]
        );
    }

    #[tokio::test]
    async fn non_retryable_status_returns_immediately() {
        let report = run(
            &[Raw::Status {
                status: 400,
                retry_after: RetryAfter::None,
            }],
            10,
        )
        .await;
        assert_eq!(report.outcome, Outcome::Status(400));
        assert_eq!(report.dispatches, 1);
        assert!(report.sleeps_ms.is_empty());
    }

    #[tokio::test]
    async fn non_retryable_transport_returns_without_redispatch() {
        // The decisive regression: a non-retryable transport class must NOT be
        // re-dispatched up to max_attempts.
        for class in [TransportErrorClass::Decode, TransportErrorClass::Builder] {
            let report = run(&[Raw::Transport(class)], 10).await;
            assert_eq!(report.outcome, Outcome::Transport(class));
            assert_eq!(report.dispatches, 1, "class {class:?} must dispatch once");
            assert!(report.sleeps_ms.is_empty());
        }
    }

    #[tokio::test]
    async fn mixed_transport_then_status_then_success() {
        let report = run(
            &[
                Raw::Transport(TransportErrorClass::Timeout),
                Raw::Status {
                    status: 429,
                    retry_after: RetryAfter::DeltaSecs(1),
                },
                Raw::Ok,
            ],
            10,
        )
        .await;
        assert_eq!(report.outcome, Outcome::Success);
        assert_eq!(report.dispatches, 3);
        assert_eq!(report.sleeps_ms, vec![50, 1000]);
    }

    #[tokio::test]
    async fn no_retry_policy_makes_one_attempt() {
        let report = run(
            &[Raw::Status {
                status: 503,
                retry_after: RetryAfter::None,
            }],
            1,
        )
        .await;
        assert_eq!(report.outcome, Outcome::Status(503));
        assert_eq!(report.dispatches, 1);
        assert!(report.sleeps_ms.is_empty());
    }
}
