#[path = "support/order_sign_submit_smoke.rs"]
mod order_sign_submit_smoke;

fn main() {
    let _ = order_sign_submit_smoke::smoke_hash_and_uid();
    let _ = cow_sdk::TradingSdk::new(
        cow_sdk::PartialTraderParameters::default(),
        cow_sdk::TradingSdkOptions::default(),
    );
}
