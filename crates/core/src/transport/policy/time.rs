//! Target-neutral async sleep helper and wall clock for retry delays.

use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::SystemTime;

#[cfg(not(target_arch = "wasm32"))]
use futures_timer::Delay;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;

/// Sleeps for the supplied retry delay on the active target.
///
/// # Panics
///
/// Panics only on `wasm32` if the explicitly clamped millisecond duration
/// cannot be represented by the timer API.
pub async fn sleep(duration: Duration) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Delay::new(duration).await;
    }

    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: clamp before converting for the wasm timer API.
        let millis = u32::try_from(duration.as_millis().min(u128::from(u32::MAX)))
            .expect("millisecond delay is clamped to `u32::MAX`");
        TimeoutFuture::new(millis).await;
    }
}

/// Returns the current wall-clock time as a [`std::time::SystemTime`] on the
/// active target.
///
/// Retry-delay computation reads the wall clock to evaluate an absolute
/// `Retry-After` HTTP-date against "now". On native targets this resolves to
/// [`std::time::SystemTime::now`]. On `wasm32-unknown-unknown` the standard
/// `SystemTime::now` is unavailable and panics, so this helper reads the
/// browser wall clock through `web_time::SystemTime` and re-anchors it onto a
/// [`std::time::SystemTime`] via `UNIX_EPOCH` arithmetic (which is available on
/// wasm). The returned value therefore feeds
/// [`crate::transport::policy::RetryPolicy::delay_for_status`] and
/// [`crate::transport::policy::parse_retry_after`] uniformly across targets without a panic path.
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn system_now() -> SystemTime {
    SystemTime::now()
}

/// Returns the current wall-clock time as a [`std::time::SystemTime`] on the
/// active target.
///
/// See the native variant for the full contract. On `wasm32-unknown-unknown`
/// the standard `SystemTime::now` panics, so this reads the browser wall clock
/// through `web_time::SystemTime` and re-anchors the elapsed duration onto a
/// [`std::time::SystemTime`] starting at `UNIX_EPOCH`. A clock reported before
/// the Unix epoch saturates to `UNIX_EPOCH`.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn system_now() -> std::time::SystemTime {
    let since_epoch = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    std::time::SystemTime::UNIX_EPOCH + since_epoch
}
