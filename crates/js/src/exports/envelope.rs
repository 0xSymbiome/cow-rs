use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

/// Version tag carried by wasm output and error envelopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum SchemaVersion {
    /// Current schema version.
    V1,
    /// Forward-compatible sentinel for schema versions unknown to this crate.
    #[serde(rename = "__unknown")]
    Unknown,
}

/// Versioned output envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct WasmEnvelope<T> {
    /// Schema version.
    pub schema_version: SchemaVersion,
    /// Envelope payload.
    pub value: T,
}

impl<T> WasmEnvelope<T> {
    /// Wraps a payload in a v1 envelope.
    pub const fn v1(value: T) -> Self {
        Self {
            schema_version: SchemaVersion::V1,
            value,
        }
    }
}
