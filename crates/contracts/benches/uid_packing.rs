use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_contracts::{OrderUidParams, extract_order_uid_params, pack_order_uid_params};
use cow_sdk_core::{Address, OrderDigest};

fn sample_params() -> OrderUidParams {
    OrderUidParams::new(
        OrderDigest::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        1_709_990_000,
    )
}

fn bench_pack_order_uid_params(c: &mut Criterion) {
    let params = sample_params();
    c.bench_function("pack_order_uid_params", |b| {
        b.iter(|| {
            let uid = pack_order_uid_params(black_box(&params))
                .expect("fixed UID parameters must pack successfully");
            black_box(uid);
        });
    });
}

fn bench_extract_order_uid_params(c: &mut Criterion) {
    let params = sample_params();
    let uid = pack_order_uid_params(&params).expect("fixed UID parameters must pack successfully");
    c.bench_function("extract_order_uid_params", |b| {
        b.iter(|| {
            let extracted = extract_order_uid_params(black_box(&uid))
                .expect("packed UID must round-trip through extract_order_uid_params");
            black_box(extracted);
        });
    });
}

criterion_group!(
    benches,
    bench_pack_order_uid_params,
    bench_extract_order_uid_params
);
criterion_main!(benches);
