use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_core::{OrderData, SupportedChainId};
use cow_sdk_signing::order_typed_data_payload;
use cow_sdk_test_utils::builders::OrderBuilder;

fn sample_order() -> OrderData {
    OrderBuilder::weth_dai()
        .receiver("0x3333333333333333333333333333333333333333")
        .app_data("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .build()
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
