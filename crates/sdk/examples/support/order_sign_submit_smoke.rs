use cow_sdk::{
    Address, Amount, AppDataHex, ORDER_PRIMARY_TYPE, OrderBalance, OrderKind,
    PartialTraderParameters, SupportedChainId, TradingSdk, TradingSdkOptions, UnsignedOrder,
    generate_order_id, order_typed_data,
};

pub fn smoke_hash_and_uid() -> Result<String, Box<dyn std::error::Error>> {
    let _sdk = TradingSdk::new(
        PartialTraderParameters::default(),
        TradingSdkOptions::default(),
    );
    let owner = Address::new("0x4444444444444444444444444444444444444444")?;
    let order = UnsignedOrder {
        sell_token: Address::new("0x1111111111111111111111111111111111111111")?,
        buy_token: Address::new("0x2222222222222222222222222222222222222222")?,
        receiver: Address::new("0x3333333333333333333333333333333333333333")?,
        sell_amount: Amount::new("100000000000000000")?,
        buy_amount: Amount::new("250000000000000000")?,
        valid_to: 1_700_000_000,
        app_data: AppDataHex::new(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )?,
        fee_amount: Amount::zero(),
        kind: OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: OrderBalance::Erc20,
        buy_token_balance: OrderBalance::Erc20,
    };

    let typed = order_typed_data(SupportedChainId::Sepolia, &order, None)?;
    let generated = generate_order_id(SupportedChainId::Sepolia, &order, &owner, None)?;

    assert_eq!(typed.primary_type, ORDER_PRIMARY_TYPE);
    Ok(generated.order_id.as_str().to_owned())
}
