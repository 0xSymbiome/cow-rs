use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_app_data::{
    AppDataParams, MetadataMap, app_data_hex_to_cid, generate_app_data_doc, stringify_deterministic,
};
use cow_sdk_core::AppCode;

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

    generate_app_data_doc(
        AppDataParams::new(AppCode::new("cow-sdk-bench").expect("bench appCode must validate"))
            .with_environment("production")
            .with_metadata(metadata),
    )
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

fn bench_app_data_hex_to_cid(c: &mut Criterion) {
    let app_data_hex = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    c.bench_function("app_data_hex_to_cid", |b| {
        b.iter(|| {
            let cid = app_data_hex_to_cid(black_box(app_data_hex))
                .expect("fixed app-data hash must convert to a CID");
            black_box(cid);
        });
    });
}

criterion_group!(
    benches,
    bench_stringify_deterministic,
    bench_app_data_hex_to_cid
);
criterion_main!(benches);
