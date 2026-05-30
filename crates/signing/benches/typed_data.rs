use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderData, OrderKind, SellTokenSource,
    SupportedChainId,
};
use cow_sdk_signing::order_typed_data_payload;

fn sample_order() -> OrderData {
    OrderData::new(
        Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(),
        Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        Amount::new("1000000000000000000").unwrap(),
        Amount::new("2000000000000000000000").unwrap(),
        1_709_990_000,
        AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        Amount::new("5000000000000000").unwrap(),
        OrderKind::Sell,
        false,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    )
}

fn bench_order_typed_data_payload(c: &mut Criterion) {
    let order = sample_order();
    c.bench_function("order_typed_data_payload", |b| {
        b.iter(|| {
            let payload = order_typed_data_payload(
                black_box(SupportedChainId::Mainnet),
                black_box(&order),
                None,
            )
            .expect("fixed order must construct a typed-data payload");
            black_box(payload);
        });
    });
}

criterion_group!(benches, bench_order_typed_data_payload);
criterion_main!(benches);
