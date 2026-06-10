use cow_sdk_core::{CowEnv, SupportedChainId};
use cow_sdk_trading::{TraderParams, Trading, TradingOptions};

fn main() {
    let params =
        TraderParams::new(SupportedChainId::Mainnet, "downstream-app").expect("app code should validate").with_env(CowEnv::Prod);

    let _sdk = Trading::new(params, TradingOptions::default());
}
