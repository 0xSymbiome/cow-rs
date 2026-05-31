use cow_sdk_core::SupportedChainId;
use cow_sdk_trading::{OrderTraderParameters, TradingBuilder};

fn order_parameters() -> OrderTraderParameters {
    unimplemented!()
}

fn main() {
    let sdk = TradingBuilder::new()
        .with_chain_id(SupportedChainId::Sepolia)
        .build_helper_only()
        .unwrap();

    let signer = ();
    let _ = sdk.off_chain_cancel_order(&order_parameters(), &signer);
}
