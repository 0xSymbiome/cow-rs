const REQUEST_SOURCE: &str = include_str!("../src/request.rs");

#[test]
fn wasm_zero_delay_path_yields_through_gloo_timeout_zero() {
    assert!(
        REQUEST_SOURCE.contains("TimeoutFuture::new(millis).await;"),
        "wasm32 delay_for must await the clamped gloo timeout value"
    );
    assert!(
        REQUEST_SOURCE.contains("duration.as_millis().min(u128::from(u32::MAX))"),
        "wasm32 delay_for must derive the gloo timeout from the requested duration"
    );
    assert!(
        !REQUEST_SOURCE.contains("if millis > 0"),
        "wasm32 delay_for must not short-circuit Duration::ZERO before yielding"
    );
    assert!(
        !REQUEST_SOURCE.contains("wasm_bindgen_futures::yield_now"),
        "wasm32 delay_for must not depend on an unpinned yield_now helper"
    );
}
