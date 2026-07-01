use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_contracts::{compute_order_uid, hash_order};
use cow_sdk_core::{Address, OrderData};
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

fn bench_compute_order_uid(c: &mut Criterion) {
    let domain = sample_domain();
    let order = sample_order();
    let owner = Address::new("0x1111111111111111111111111111111111111111")
        .expect("bench owner address must validate");
    c.bench_function("compute_order_uid", |b| {
        b.iter(|| {
            let uid = compute_order_uid(black_box(&domain), black_box(&order), black_box(&owner));
            black_box(uid);
        });
    });
}

criterion_group!(benches, bench_hash_order, bench_compute_order_uid);
criterion_main!(benches);
