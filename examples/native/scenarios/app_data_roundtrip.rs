use std::error::Error;

use serde_json::json;

use cow_sdk::{
    AppDataParams, CidMode, SchemaVersion, app_data_hex_to_cid_with_mode, cid_to_app_data_hex,
    generate_app_data_doc, get_app_data_info, get_app_data_info_legacy, get_app_data_schema,
    validate_app_data_doc,
};

fn main() -> Result<(), Box<dyn Error>> {
    let document = generate_app_data_doc(AppDataParams {
        app_code: Some("cow-rs/app-data-roundtrip".to_owned()),
        environment: Some("example".to_owned()),
        ..Default::default()
    });
    let validation = validate_app_data_doc(&document);
    let current = get_app_data_info(&document)?;
    let legacy = get_app_data_info_legacy(&document)?;
    let latest_cid = app_data_hex_to_cid_with_mode(&current.app_data_hex, CidMode::Latest)?;
    let legacy_cid = app_data_hex_to_cid_with_mode(&legacy.app_data_hex, CidMode::Legacy)?;
    let schema = get_app_data_schema(SchemaVersion::latest().as_str())?;

    let report = json!({
        "surface": "cow-sdk::app_data",
        "mode": "deterministic",
        "valid": validation.success,
        "schemaVersion": SchemaVersion::latest().as_str(),
        "schemaType": schema.get("type").and_then(|value| value.as_str()),
        "current": {
            "cid": current.cid,
            "appDataHex": current.app_data_hex,
            "cidRoundtripHex": cid_to_app_data_hex(&latest_cid)?,
        },
        "legacy": {
            "cid": legacy.cid,
            "appDataHex": legacy.app_data_hex,
            "cidRoundtripHex": cid_to_app_data_hex(&legacy_cid)?,
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
