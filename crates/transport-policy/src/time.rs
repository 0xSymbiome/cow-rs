//! Target-neutral async sleep helper for retry delays.

use std::time::Duration;

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
        let millis = u32::try_from(duration.as_millis().min(u128::from(u32::MAX)))
            .expect("millisecond delay is clamped to `u32::MAX`");
        TimeoutFuture::new(millis).await;
    }
}
