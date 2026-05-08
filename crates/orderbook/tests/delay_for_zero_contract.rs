const TIME_SOURCE: &str = include_str!("../../transport-policy/src/time.rs");

#[test]
fn wasm_zero_delay_path_yields_through_gloo_timeout_zero() {
    assert!(
        TIME_SOURCE.contains("TimeoutFuture::new(millis).await;"),
        "wasm32 retry sleep must await the clamped gloo timeout value"
    );
    assert!(
        TIME_SOURCE.contains("duration.as_millis().min(u128::from(u32::MAX))"),
        "wasm32 retry sleep must derive the gloo timeout from the requested duration"
    );
    assert!(
        !TIME_SOURCE.contains("if millis > 0"),
        "wasm32 retry sleep must not short-circuit Duration::ZERO before yielding"
    );
    assert!(
        !TIME_SOURCE.contains("wasm_bindgen_futures::yield_now"),
        "wasm32 retry sleep must not depend on an unpinned yield_now helper"
    );
}
