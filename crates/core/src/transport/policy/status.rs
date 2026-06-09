//! Retryable HTTP status constants.

/// HTTP `408 Request Timeout`.
pub const REQUEST_TIMEOUT: u16 = 408;
/// HTTP `425 Too Early`.
pub const TOO_EARLY: u16 = 425;
/// HTTP `429 Too Many Requests`.
pub const TOO_MANY_REQUESTS: u16 = 429;
/// HTTP `500 Internal Server Error`.
pub const INTERNAL_SERVER_ERROR: u16 = 500;
/// HTTP `502 Bad Gateway`.
pub const BAD_GATEWAY: u16 = 502;
/// HTTP `503 Service Unavailable`.
pub const SERVICE_UNAVAILABLE: u16 = 503;
/// HTTP `504 Gateway Timeout`.
pub const GATEWAY_TIMEOUT: u16 = 504;

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
    matches!(
        status,
        REQUEST_TIMEOUT
            | TOO_EARLY
            | TOO_MANY_REQUESTS
            | INTERNAL_SERVER_ERROR
            | BAD_GATEWAY
            | SERVICE_UNAVAILABLE
            | GATEWAY_TIMEOUT
    )
}
