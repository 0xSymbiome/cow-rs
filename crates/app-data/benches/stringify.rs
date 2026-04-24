use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_app_data::{
    AppDataParams, MetadataMap, generate_app_data_doc, stringify_deterministic,
};

fn sample_app_data() -> cow_sdk_app_data::AppDataDoc {
    let mut metadata = MetadataMap::new();
    metadata.insert(
        "orderClass".to_owned(),
        serde_json::json!({ "orderClass": "market" }),
    );
    metadata.insert(
        "quote".to_owned(),
        serde_json::json!({ "slippageBips": 50 }),
    );

    generate_app_data_doc(AppDataParams::new(
        Some("cow-sdk-bench".to_owned()),
        Some("production".to_owned()),
        None,
        None,
        metadata,
    ))
}

fn bench_stringify_deterministic(c: &mut Criterion) {
    let doc = sample_app_data();
    c.bench_function("stringify_deterministic", |b| {
        b.iter(|| {
            let rendered = stringify_deterministic(black_box(&doc))
                .expect("fixed app-data document must render deterministically");
            black_box(rendered);
        });
    });
}

criterion_group!(benches, bench_stringify_deterministic);
criterion_main!(benches);
