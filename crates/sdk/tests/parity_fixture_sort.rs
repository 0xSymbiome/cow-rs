use std::path::Path;

#[test]
fn sdk_parity_fixture_cases_remain_sorted_by_id() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("parity/fixtures/sdk.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} must be readable: {error}", path.display()));
    let fixture: serde_json::Value =
        serde_json::from_str(&raw).expect("sdk fixture must parse as JSON");
    let cases = fixture["cases"]
        .as_array()
        .expect("sdk fixture cases must be an array");
    let ids = cases
        .iter()
        .map(|case| {
            case["id"]
                .as_str()
                .expect("sdk fixture case id must be a string")
        })
        .collect::<Vec<_>>();
    let mut sorted = ids.clone();
    sorted.sort_unstable();

    assert_eq!(ids, sorted, "sdk fixture cases must be sorted by id");
}
