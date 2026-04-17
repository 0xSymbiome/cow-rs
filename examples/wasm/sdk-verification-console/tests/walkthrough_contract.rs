use cow_sdk_verification_console::walkthrough_determinism_cycle_json;
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn walkthrough_determinism_cycle_drives_reviewed_helpers_in_order() {
    let envelope = parse_json(walkthrough_determinism_cycle_json());

    assert_eq!(envelope["name"], "sdk-verification-console.determinism-cycle");
    assert_eq!(envelope["completed"], true);
    assert!(
        envelope["failedAt"].is_null(),
        "deterministic walkthrough must never fail"
    );

    let steps = envelope["steps"]
        .as_array()
        .expect("walkthrough envelope must expose a steps array");

    let names: Vec<&str> = steps
        .iter()
        .map(|step| step["name"].as_str().expect("step name must be a string"))
        .collect();

    assert_eq!(
        names,
        vec![
            "supported-chains",
            "capability-report",
            "app-data-report",
            "hex-from-cid",
            "cid-from-hex",
        ],
        "walkthrough step order must stay stable"
    );

    for step in steps {
        assert!(
            step["result"].is_array() || step["result"].is_object(),
            "each walkthrough step must carry a reviewable JSON result"
        );
    }

    let capability = steps
        .iter()
        .find(|step| step["name"] == "capability-report")
        .expect("capability-report step must be present");
    assert_eq!(capability["result"]["surface"], "cow-sdk");
    assert_eq!(capability["result"]["mode"], "wasm-console");
    assert_eq!(capability["result"]["chainId"], 1);
    assert_eq!(capability["result"]["sdkConstructed"], true);
}

fn parse_json(result: Result<String, JsValue>) -> Value {
    serde_json::from_str(&result.expect("walkthrough export must return JSON"))
        .expect("walkthrough JSON must parse")
}
