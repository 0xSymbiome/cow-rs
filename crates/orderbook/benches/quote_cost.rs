use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_orderbook::calculate_total_fee;

fn bench_calculate_total_fee(c: &mut Criterion) {
    let executed_fee = "1500000000000000000";
    c.bench_function("calculate_total_fee", |b| {
        b.iter(|| {
            let total = calculate_total_fee(black_box(Some(executed_fee)))
                .expect("decimal input must normalize without overflow");
            black_box(total);
        });
    });
}

criterion_group!(benches, bench_calculate_total_fee);
criterion_main!(benches);
