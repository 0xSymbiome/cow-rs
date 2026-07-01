use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_contracts::{RecoverableSignature, SigningScheme, hash_order};
use cow_sdk_core::OrderData;
use cow_sdk_test_utils::builders::{OrderBuilder, sample_domain};
use cow_sdk_test_utils::consts::EXPECTED_ORDER_SIGNATURE;

fn sample_order() -> OrderData {
    OrderBuilder::weth_dai()
        .receiver("0x3333333333333333333333333333333333333333")
        .app_data("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        .build()
}

fn bench_recover(c: &mut Criterion) {
    let signature = RecoverableSignature::parse_hex(EXPECTED_ORDER_SIGNATURE)
        .expect("canonical signature must parse");
    let digest = hash_order(&sample_domain(), &sample_order());
    c.bench_function("recover", |b| {
        b.iter(|| {
            let signer = signature
                .recover(black_box(&digest), black_box(SigningScheme::Eip712))
                .expect("canonical signature must recover a signer");
            black_box(signer);
        });
    });
}

criterion_group!(benches, bench_recover);
criterion_main!(benches);
