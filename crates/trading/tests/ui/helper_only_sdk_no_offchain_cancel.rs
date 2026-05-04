use cow_sdk_core::SupportedChainId;
use cow_sdk_trading::{OrderTraderParameters, TradingSdkBuilder};

fn order_parameters() -> OrderTraderParameters {
    unimplemented!()
}

fn main() {
    let sdk = TradingSdkBuilder::new()
        .with_chain_id(SupportedChainId::Sepolia)
        .build_helper_only()
        .unwrap();

    let signer = ();
    let _ = sdk.off_chain_cancel_order_async(&order_parameters(), &signer);
}
