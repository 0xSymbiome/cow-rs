use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, SellTokenSource, SupportedChainId,
};
use cow_sdk_trading::{LimitTradeParameters, OrderToSignParams, get_order_to_sign};

fn sample_limit_parameters() -> LimitTradeParameters {
    LimitTradeParameters::new(
        OrderKind::Sell,
        Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(),
        18,
        Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        18,
        Amount::new("1000000000000000000").unwrap(),
        Amount::new("2000000000000000000000").unwrap(),
    )
    .with_sell_token_balance(SellTokenSource::Erc20)
    .with_buy_token_balance(BuyTokenDestination::Erc20)
    .with_slippage_bps(50)
    .with_valid_to(1_900_000_000)
}

fn sample_params() -> OrderToSignParams {
    OrderToSignParams::new(
        SupportedChainId::Mainnet,
        Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        false,
    )
    .with_apply_costs_slippage_and_fees(false)
}

fn bench_get_order_to_sign(c: &mut Criterion) {
    let parameters = sample_limit_parameters();
    let params = sample_params();
    let app_data_hash =
        AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap();
    c.bench_function("get_order_to_sign", |b| {
        b.iter(|| {
            let order = get_order_to_sign(
                black_box(params),
                black_box(&parameters),
                black_box(&app_data_hash),
            )
            .expect("fixed limit parameters must resolve to a signed-order payload");
            black_box(order);
        });
    });
}

criterion_group!(benches, bench_get_order_to_sign);
criterion_main!(benches);
