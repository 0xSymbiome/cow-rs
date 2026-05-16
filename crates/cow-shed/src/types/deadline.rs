use std::time::Duration;

use alloy_primitives::U256;

/// Deadline strategy for COW Shed hook authorization.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Deadline {
    /// No practical expiry.
    Never,
    /// Absolute UNIX timestamp in seconds.
    Absolute(u64),
    /// Relative duration from the supplied `now` timestamp.
    Relative(Duration),
}

impl Deadline {
    /// Resolves this deadline to the `uint256` value encoded into calldata.
    #[must_use]
    pub fn resolve(self, now: u64) -> U256 {
        match self {
            Self::Never => U256::MAX,
            Self::Absolute(value) => U256::from(value),
            Self::Relative(duration) => U256::from(now.saturating_add(duration.as_secs())),
        }
    }
}
