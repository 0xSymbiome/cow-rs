use std::fmt;

use serde::{Deserialize, Serialize};

use crate::errors::{CoreError, ValidationError};
/// Minimum relative-window duration accepted by [`ValidTo::relative`], in seconds.
pub const VALID_TO_MIN_RELATIVE_SECONDS: u32 = 30;

/// Maximum relative-window duration accepted by [`ValidTo::relative`], in seconds.
///
/// The default ceiling of 90 days matches the longest order horizon the
/// orderbook accepts today and keeps typed construction ahead of the
/// server-side 422 response path.
pub const VALID_TO_MAX_RELATIVE_SECONDS: u32 = 90 * 24 * 60 * 60;

/// Validated order expiration timestamp encoded as a UNIX epoch in seconds.
///
/// `ValidTo` guards construction of order-deadline values so relative durations
/// that would produce an instantly-expired order or run past the orderbook's
/// accepted horizon fail closed with a typed
/// [`ValidationError::ValidToOutOfRange`] at the client boundary. Absolute
/// epochs that already fit the `u32` range are accepted as-is so existing
/// orderbook quote responses continue to round-trip without additional
/// validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValidTo(u32);

impl ValidTo {
    /// Creates a [`ValidTo`] from an absolute UNIX epoch timestamp in seconds.
    #[inline]
    #[must_use]
    pub const fn absolute(epoch_seconds: u32) -> Self {
        Self(epoch_seconds)
    }

    /// Creates a [`ValidTo`] by adding a relative window to the supplied UNIX epoch anchor.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::ValidToOutOfRange`] when the window falls
    /// outside the inclusive `[VALID_TO_MIN_RELATIVE_SECONDS,
    /// VALID_TO_MAX_RELATIVE_SECONDS]` range.
    pub fn relative(now_epoch_seconds: u64, duration_seconds: u64) -> Result<Self, CoreError> {
        if duration_seconds < u64::from(VALID_TO_MIN_RELATIVE_SECONDS)
            || duration_seconds > u64::from(VALID_TO_MAX_RELATIVE_SECONDS)
        {
            return Err(ValidationError::ValidToOutOfRange {
                actual_seconds: duration_seconds,
                min: VALID_TO_MIN_RELATIVE_SECONDS,
                max: VALID_TO_MAX_RELATIVE_SECONDS,
            }
            .into());
        }

        let projected = now_epoch_seconds.saturating_add(duration_seconds);
        let clamped = projected.min(u64::from(u32::MAX));
        u32::try_from(clamped).map(Self).map_err(|_| {
            ValidationError::ValidToOutOfRange {
                actual_seconds: duration_seconds,
                min: VALID_TO_MIN_RELATIVE_SECONDS,
                max: VALID_TO_MAX_RELATIVE_SECONDS,
            }
            .into()
        })
    }

    /// Returns the validated absolute UNIX epoch timestamp.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Returns the validated absolute UNIX epoch timestamp as a `u64`.
    #[inline]
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }
}

impl From<ValidTo> for u32 {
    #[inline]
    fn from(value: ValidTo) -> Self {
        value.0
    }
}

impl From<u32> for ValidTo {
    #[inline]
    fn from(value: u32) -> Self {
        Self::absolute(value)
    }
}

impl fmt::Display for ValidTo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
