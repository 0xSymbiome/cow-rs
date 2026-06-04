use std::error::Error;

use serde_json::json;

use cow_sdk::AppCode;
use cow_sdk::app_data::{
    AppDataParams, SchemaVersion, app_data_hex_to_cid, cid_to_app_data_hex, generate_app_data_doc,
    get_app_data_info, validate_app_data_doc,
};

fn main() -> Result<(), Box<dyn Error>> {
    let app_code = AppCode::new("cow-rs/app-data-roundtrip")?;
    let document = generate_app_data_doc(
        AppDataParams::new(app_code).with_environment("example"),
    );
    let validation = validate_app_data_doc(&document);
    let info = get_app_data_info(&document)?;
    let derived_cid = app_data_hex_to_cid(&info.app_data_hex)?;

    let report = json!({
        "surface": "cow-sdk::app_data",
        "mode": "deterministic",
        "valid": validation.success,
        "schemaVersion": SchemaVersion::latest().as_str(),
        "current": {
            "cid": info.cid,
            "appDataHex": info.app_data_hex,
            "cidRoundtripHex": cid_to_app_data_hex(&derived_cid)?,
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
