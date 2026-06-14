//! Retryable HTTP status constants.

// Individual status scalars are crate-internal: they exist to build the
// published `RETRYABLE_STATUSES` set and to name the two `Retry-After`-bearing
// codes inside the retry policy. Consumers test retryability through
// `is_retryable_status` / `RETRYABLE_STATUSES`, and read raw status codes from
// `http::StatusCode`, so the SDK does not re-export `http`'s numeric constants.
/// HTTP `408 Request Timeout`.
pub(crate) const REQUEST_TIMEOUT: u16 = 408;
/// HTTP `425 Too Early`.
pub(crate) const TOO_EARLY: u16 = 425;
/// HTTP `429 Too Many Requests`.
pub(crate) const TOO_MANY_REQUESTS: u16 = 429;
/// HTTP `500 Internal Server Error`.
pub(crate) const INTERNAL_SERVER_ERROR: u16 = 500;
/// HTTP `502 Bad Gateway`.
pub(crate) const BAD_GATEWAY: u16 = 502;
/// HTTP `503 Service Unavailable`.
pub(crate) const SERVICE_UNAVAILABLE: u16 = 503;
/// HTTP `504 Gateway Timeout`.
pub(crate) const GATEWAY_TIMEOUT: u16 = 504;

/// Status codes retried by the default SDK transport policy.
pub const RETRYABLE_STATUSES: [u16; 7] = [
    REQUEST_TIMEOUT,
    TOO_EARLY,
    TOO_MANY_REQUESTS,
    INTERNAL_SERVER_ERROR,
    BAD_GATEWAY,
    SERVICE_UNAVAILABLE,
    GATEWAY_TIMEOUT,
];

/// Returns whether `status` is retried by the default SDK transport policy.
#[must_use]
pub const fn is_retryable_status(status: u16) -> bool {
    // Loop over the single `RETRYABLE_STATUSES` source so the predicate cannot
    // drift from the published list. `[u16]::contains` is not yet `const`.
    let mut index = 0;
    while index < RETRYABLE_STATUSES.len() {
        if RETRYABLE_STATUSES[index] == status {
            return true;
        }
        index += 1;
    }
    false
}
