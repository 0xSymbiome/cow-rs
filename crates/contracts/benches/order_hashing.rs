use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_contracts::hash_order;
use cow_sdk_core::OrderData;
use cow_sdk_test_utils::builders::{OrderBuilder, sample_domain};

fn sample_order() -> OrderData {
    OrderBuilder::weth_dai()
        .receiver("0x3333333333333333333333333333333333333333")
        .app_data("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .build()
}

fn bench_hash_order(c: &mut Criterion) {
    let domain = sample_domain();
    let order = sample_order();
    c.bench_function("hash_order", |b| {
        b.iter(|| {
            let digest = hash_order(black_box(&domain), black_box(&order));
            black_box(digest);
        });
    });
}

criterion_group!(benches, bench_hash_order);
criterion_main!(benches);
