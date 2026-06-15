//! The error type the doubles return on injected failure, plus ready-made
//! orderbook-error constructors and the injectable [`OrderbookFailure`] kinds.

use cow_sdk_orderbook::{OrderbookApiError, OrderbookError, ResponseBody};

/// Error returned by [`MockSigner`](crate::MockSigner) and
/// [`MockProvider`](crate::MockProvider) when a consumer injects a failure.
///
/// It implements [`std::error::Error`], [`std::fmt::Display`], and
/// [`cow_sdk_core::UserRejection`], so it satisfies the bounds the SDK's signer
/// and provider helper surfaces place on an implementation's `Error` type.
#[derive(Debug, Clone)]
pub struct MockError(String);

impl MockError {
    /// Builds an error carrying `message`.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    /// Returns the carried message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for MockError {}

// Default `user_rejection_code` (`None`) is the correct posture for a test
// double: an injected failure is not a real EIP-1193 wallet rejection.
impl cow_sdk_core::UserRejection for MockError {}

/// A canned orderbook failure a consumer can inject into a
/// [`MockOrderbook`](crate::MockOrderbook) builder.
///
/// `OrderbookError` is not `Clone`, so the doubles store this small cloneable
/// description and rebuild a fresh error on each call instead.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum OrderbookFailure {
    /// A 404 not-found rejection (an unknown order or quote).
    NotFound,
    /// A 429 rate-limited rejection.
    RateLimited,
    /// A generic rejection carrying a message.
    Rejected(String),
}

impl OrderbookFailure {
    /// Builds a fresh [`OrderbookError`] for this failure kind.
    #[must_use]
    pub fn to_error(&self) -> OrderbookError {
        match self {
            Self::NotFound => order_not_found(),
            Self::RateLimited => rate_limited(),
            Self::Rejected(message) => rejected(message.clone()),
        }
    }
}

/// A 404 order-not-found orderbook error.
#[must_use]
pub fn order_not_found() -> OrderbookError {
    OrderbookApiError::new(
        404,
        "Not Found",
        ResponseBody::Text("order not found".to_owned()),
    )
    .into()
}

/// A 429 rate-limited orderbook error.
#[must_use]
pub fn rate_limited() -> OrderbookError {
    OrderbookApiError::new(
        429,
        "Too Many Requests",
        ResponseBody::Text("rate limit exceeded".to_owned()),
    )
    .into()
}

/// A generic 400 rejection orderbook error carrying `message`.
#[must_use]
pub fn rejected(message: impl Into<String>) -> OrderbookError {
    OrderbookApiError::new(400, "Bad Request", ResponseBody::Text(message.into())).into()
}
