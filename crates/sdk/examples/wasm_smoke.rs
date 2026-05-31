#[path = "support/order_sign_submit_smoke.rs"]
mod order_sign_submit_smoke;

fn main() {
    let _ = order_sign_submit_smoke::smoke_hash_and_uid();
    let _ = cow_sdk::TradingBuilder::helper_only(
        cow_sdk::core::SupportedChainId::Sepolia,
        cow_sdk::trading::TradingOptions::default(),
    )
    .expect("helper-only trading sdk construction should succeed");
}
