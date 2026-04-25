#[path = "support/order_sign_submit_smoke.rs"]
mod order_sign_submit_smoke;

fn main() {
    let _ = order_sign_submit_smoke::smoke_hash_and_uid();
    let _ = cow_sdk::TradingSdk::new_partial(
        cow_sdk::trading::PartialTraderParameters::new()
            .with_chain_id(cow_sdk::core::SupportedChainId::Sepolia),
        cow_sdk::trading::TradingSdkOptions::default(),
    )
    .expect("partial trading sdk construction should succeed");
}
