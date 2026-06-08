//! App-data document generation, validation, and CID round-trip.
//!
//! Generates an app-data document (`generate_app_data_doc`), validates it
//! (`validate_app_data_doc`), inspects it (`app_data_info`), and round-trips
//! the content hash through its IPFS CID (`app_data_hex_to_cid` /
//! `cid_to_app_data_hex`). Pure codec — no transport.

use std::error::Error;

use serde_json::json;

use cow_sdk::app_data::{
    AppDataParams, SchemaVersion, app_data_hex_to_cid, app_data_info, cid_to_app_data_hex,
    generate_app_data_doc, validate_app_data_doc,
};
use cow_sdk::core::AppCode;

fn main() -> Result<(), Box<dyn Error>> {
    // Generate an app-data document from typed params (generation is infallible).
    let app_code = AppCode::new("cow-rs/app-data-roundtrip")?;
    let document = generate_app_data_doc(AppDataParams::new(app_code).with_environment("example"));

    // Validate the generated JSON against the app-data schema.
    let validation = validate_app_data_doc(&document);

    // Inspect the document: its canonical content hash and the IPFS CID it pins to.
    let info = app_data_info(&document)?;

    // Re-derive the CID from the hash; the report below maps it back with
    // `cid_to_app_data_hex` to show the conversion is lossless.
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
