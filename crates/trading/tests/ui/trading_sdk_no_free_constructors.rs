use cow_sdk_core::{CowEnv, SupportedChainId};
use cow_sdk_trading::{TraderParameters, TradingSdk, TradingSdkOptions};

fn main() {
    let params =
        TraderParameters::new(SupportedChainId::Mainnet, "downstream-app").expect("app code should validate").with_env(CowEnv::Prod);

    let _sdk = TradingSdk::new(params, TradingSdkOptions::default());
}
