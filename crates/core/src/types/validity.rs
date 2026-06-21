use std::fmt;

use serde::{Deserialize, Serialize};

use crate::errors::{CoreError, ValidationError};

/// Validated order expiration timestamp encoded as a UNIX epoch in seconds.
///
/// `ValidTo` keeps order-deadline values inside the protocol-fixed `u32` epoch
/// range (the `MAX_VALID_TO_EPOCH` ceiling, year 2106). It does not bake an
/// operator-tunable validity window: per ADR 0015 the exact minimum/maximum
/// order-validity policy is the orderbook's, so the client mirrors only the
/// protocol-fixed range and lets the server own the tunable window. Absolute
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

    /// Creates a [`ValidTo`] by adding a relative duration to a UNIX epoch anchor.
    ///
    /// The anchor and duration are added with saturating arithmetic; the result
    /// fails closed only against the protocol-fixed `u32` epoch ceiling, leaving
    /// the operator-tunable validity window to the orderbook (ADR 0015).
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::ValidToOutOfRange`] when the resulting absolute
    /// timestamp exceeds the protocol-fixed `u32` epoch ceiling.
    pub fn relative(now_epoch_seconds: u64, duration_seconds: u64) -> Result<Self, CoreError> {
        let projected = now_epoch_seconds.saturating_add(duration_seconds);
        u32::try_from(projected).map(Self).map_err(|_| {
            ValidationError::ValidToOutOfRange {
                actual_seconds: projected,
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
