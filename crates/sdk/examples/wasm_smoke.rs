#[path = "support/order_sign_submit_smoke.rs"]
mod order_sign_submit_smoke;

fn main() {
    let _ = order_sign_submit_smoke::smoke_hash_and_uid();
    let _ = cow_sdk::TradingSdk::new_partial(
        cow_sdk::trading::PartialTraderParameters::default(),
        cow_sdk::trading::TradingSdkOptions::default(),
    )
    .expect("default partial trading sdk construction should succeed");
}
