use cow_sdk::core::{AppDataHex, BuyTokenDestination, OrderKind, SellTokenSource, UnsignedOrder};
use cow_sdk::prelude::{Address, Amount, SupportedChainId, TradingSdkBuilder};
use cow_sdk::signing::{ORDER_PRIMARY_TYPE, generate_order_id, order_typed_data};
use cow_sdk::trading::TradingSdkOptions;

pub fn smoke_hash_and_uid() -> Result<String, Box<dyn std::error::Error>> {
    let _sdk =
        TradingSdkBuilder::helper_only(SupportedChainId::Sepolia, TradingSdkOptions::default())?;
    let owner = Address::new("0x4444444444444444444444444444444444444444")?;
    let order = UnsignedOrder::new(
        Address::new("0x1111111111111111111111111111111111111111")?,
        Address::new("0x2222222222222222222222222222222222222222")?,
        Address::new("0x3333333333333333333333333333333333333333")?,
        Amount::new("100000000000000000")?,
        Amount::new("250000000000000000")?,
        1_700_000_000,
        AppDataHex::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")?,
        Amount::zero(),
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    );

    let typed = order_typed_data(SupportedChainId::Sepolia, &order, None)?;
    let generated = generate_order_id(SupportedChainId::Sepolia, &order, &owner, None)?;

    assert_eq!(typed.primary_type, ORDER_PRIMARY_TYPE);
    Ok(generated.order_id.as_str().to_owned())
}
