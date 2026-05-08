//! Retry jitter strategies.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_JITTER_WINDOW_DIVISOR: u32 = 2;

/// Jitter policy applied to retry backoff delays.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JitterStrategy {
    /// Leave retry delays unchanged.
    None,
    /// Pick a delay uniformly across the full base-delay window.
    Full { seed: u64 },
    /// Preserve half of the base delay and jitter the remaining half.
    Equal { seed: u64 },
    /// Add a deterministic decorrelated offset bounded by half the base delay.
    Decorrelated { seed: u64 },
}

impl JitterStrategy {
    /// Returns a strategy that leaves retry delays unchanged.
    #[must_use]
    pub const fn none() -> Self {
        Self::None
    }

    /// Returns a full-jitter strategy with a time-derived seed.
    #[must_use]
    pub fn full() -> Self {
        Self::full_from_seed(jitter_seed())
    }

    /// Returns a full-jitter strategy with a caller-supplied seed.
    #[must_use]
    pub const fn full_from_seed(seed: u64) -> Self {
        Self::Full { seed }
    }

    /// Returns an equal-jitter strategy with a time-derived seed.
    #[must_use]
    pub fn equal() -> Self {
        Self::equal_from_seed(jitter_seed())
    }

    /// Returns an equal-jitter strategy with a caller-supplied seed.
    #[must_use]
    pub const fn equal_from_seed(seed: u64) -> Self {
        Self::Equal { seed }
    }

    /// Returns the default decorrelated retry jitter strategy.
    #[must_use]
    pub fn decorrelated() -> Self {
        Self::decorrelated_from_seed(jitter_seed())
    }

    /// Returns a decorrelated strategy with a caller-supplied seed.
    #[must_use]
    pub const fn decorrelated_from_seed(seed: u64) -> Self {
        Self::Decorrelated { seed }
    }

    /// Applies jitter to `base_delay` for `attempt_index`, bounded by `max_delay`.
    #[must_use]
    pub fn delay_for_attempt(
        self,
        base_delay: Duration,
        max_delay: Duration,
        attempt_index: usize,
    ) -> Duration {
        let capped_base = base_delay.min(max_delay);
        let delay = match self {
            Self::None => capped_base,
            Self::Full { seed } => bounded_offset(seed, attempt_index, capped_base),
            Self::Equal { seed } => {
                let half = capped_base / 2;
                half.saturating_add(bounded_offset(seed, attempt_index, half))
            }
            Self::Decorrelated { seed } => {
                let window = capped_base / DEFAULT_JITTER_WINDOW_DIVISOR;
                capped_base.saturating_add(bounded_offset(seed, attempt_index, window))
            }
        };
        delay.min(max_delay)
    }
}

impl Default for JitterStrategy {
    fn default() -> Self {
        Self::decorrelated()
    }
}

fn bounded_offset(seed: u64, attempt_index: usize, window: Duration) -> Duration {
    let window_ms = window.as_millis();
    if window_ms == 0 {
        return Duration::ZERO;
    }
    let bounded_window = window_ms.saturating_add(1).min(u128::from(u64::MAX));
    let attempt = u64::try_from(attempt_index).unwrap_or(u64::MAX);
    let offset = u128::from(splitmix64(seed ^ attempt)) % bounded_window;
    Duration::from_millis(u64::try_from(offset).expect("jitter offset is capped to u64"))
}

fn jitter_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0x9E37_79B9_7F4A_7C15, |duration| {
            duration.as_secs().rotate_left(32) ^ u64::from(duration.subsec_nanos())
        })
}

const fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}
