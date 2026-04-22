//! Narrow regression that locks the typed `PartnerFee` construction
//! on the `trading_defaults_json` export so the reviewed validator
//! round-trip cannot silently drift again.

use cow_sdk_verification_console::trading_defaults_json;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn trading_defaults_json_composes_typed_partner_fee() {
    trading_defaults_json()
        .expect("trading defaults json composes the typed partner-fee payload");
}
