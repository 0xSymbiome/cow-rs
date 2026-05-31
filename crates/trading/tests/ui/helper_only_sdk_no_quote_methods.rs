use cow_sdk_core::SupportedChainId;
use cow_sdk_trading::{TradeParameters, TradingBuilder};

fn trade_parameters() -> TradeParameters {
    unimplemented!()
}

fn main() {
    let sdk = TradingBuilder::new()
        .with_chain_id(SupportedChainId::Sepolia)
        .build_helper_only()
        .unwrap();

    let _ = sdk.get_quote_only(trade_parameters(), None);
}
