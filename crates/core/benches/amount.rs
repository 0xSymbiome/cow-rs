use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_core::Amount;

fn bench_amount_new_decimal(c: &mut Criterion) {
    let decimal = "1000000000000000000";
    c.bench_function("amount_new_decimal", |b| {
        b.iter(|| {
            let amount = Amount::new(black_box(decimal)).expect("fixed decimal amount must parse");
            black_box(amount);
        });
    });
}

criterion_group!(benches, bench_amount_new_decimal);
criterion_main!(benches);
