use wasm_bindgen::prelude::*;

use cow_sdk_pure_helpers as pure;

use crate::exports::{
    dto::{
        AppDataDocDto, AppDataDocInput, AppDataInfoDto, DeploymentAddressesDto,
        GeneratedOrderUidDto, TypedDataEnvelopeDto, ValidationResultDto, parse_chain, parse_order,
        parse_owner, to_js_value,
    },
    envelope::WasmEnvelope,
    errors::WasmError,
};

/// Computes the EIP-712 domain separator for a supported chain.
#[wasm_bindgen(js_name = "domainSeparator")]
pub fn domain_separator(
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
) -> Result<String, JsValue> {
    pure::chains::domain_separator(chain_id).map_err(|error| WasmError::from(error).into_js())
}

/// Builds signer-facing order typed data.
#[wasm_bindgen(
    js_name = "orderTypedData",
    unchecked_return_type = "WasmEnvelope<TypedDataEnvelopeDto>"
)]
pub fn order_typed_data(
    input: OrderInput,
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
) -> Result<JsValue, JsValue> {
    let order = parse_order(input)?;
    let chain = parse_chain(chain_id)?;
    let payload = pure::signing::order_typed_data_payload(chain, &order)
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(TypedDataEnvelopeDto::from_payload(
        &payload,
    )?))
}

/// Computes the compact order UID and digest.
#[wasm_bindgen(
    js_name = "computeOrderUid",
    unchecked_return_type = "WasmEnvelope<GeneratedOrderUidDto>"
)]
pub fn compute_order_uid(
    input: OrderInput,
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
    owner: String,
) -> Result<JsValue, JsValue> {
    let order = parse_order(input)?;
    let chain = parse_chain(chain_id)?;
    let owner = parse_owner(&owner)?;
    let generated = pure::signing::generate_order_id(chain, &order, &owner)
        .map_err(|error| WasmError::from(error).into_js())?;
    let dto = GeneratedOrderUidDto::from(pure::uid::generated_order_uid_dto(&generated));
    to_js_value(&WasmEnvelope::v1(dto))
}

/// Returns supported EVM chain ids.
#[wasm_bindgen(js_name = "supportedChainIds")]
#[must_use]
pub fn supported_chain_ids() -> Vec<u32> {
    pure::chains::supported_chain_ids()
}

/// Returns canonical deployment addresses for a chain and environment.
#[wasm_bindgen(
    js_name = "deploymentAddresses",
    unchecked_return_type = "WasmEnvelope<DeploymentAddressesDto>"
)]
pub fn deployment_addresses(
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
    env: Option<String>,
) -> Result<JsValue, JsValue> {
    let addresses = pure::chains::deployment_addresses(chain_id, env.as_deref())
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(DeploymentAddressesDto::from(addresses)))
}

/// Returns deterministic app-data content, hash, and CID.
#[wasm_bindgen(
    js_name = "appDataInfo",
    unchecked_return_type = "WasmEnvelope<AppDataInfoDto>"
)]
pub fn app_data_info(doc: AppDataDocInput) -> Result<JsValue, JsValue> {
    let document = pure::app_data::document_from_input(doc.into())
        .map_err(|error| WasmError::from(error).into_js())?;
    let info = pure::app_data::app_data_info(&document)
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(AppDataInfoDto::from(
        pure::dto::AppDataInfoDto::from(info),
    )))
}

/// Validates an app-data document against the embedded schemas.
#[wasm_bindgen(
    js_name = "validateAppDataDoc",
    unchecked_return_type = "WasmEnvelope<ValidationResultDto>"
)]
pub fn validate_app_data_doc(doc: AppDataDocInput) -> Result<JsValue, JsValue> {
    let document = pure::app_data::document_from_input(doc.into())
        .map_err(|error| WasmError::from(error).into_js())?;
    let result = pure::app_data::validate_app_data_doc(&document);
    to_js_value(&WasmEnvelope::v1(ValidationResultDto::from(
        pure::dto::ValidationResultDto::from(result),
    )))
}

/// Builds an app-data document without hashing it.
#[wasm_bindgen(
    js_name = "appDataDoc",
    unchecked_return_type = "WasmEnvelope<AppDataDocDto>"
)]
pub fn app_data_doc(doc: AppDataDocInput) -> Result<JsValue, JsValue> {
    let document = pure::app_data::document_from_input(doc.into())
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(AppDataDocDto::from(document)))
}

/// Converts an app-data hash to an IPFS CID.
#[wasm_bindgen(
    js_name = "appDataHexToCid",
    unchecked_return_type = "WasmEnvelope<string>"
)]
pub fn app_data_hex_to_cid(
    #[wasm_bindgen(js_name = appDataHex)] app_data_hex: String,
) -> Result<JsValue, JsValue> {
    let cid = pure::app_data::app_data_hex_to_cid(&app_data_hex)
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(cid))
}

/// Converts an IPFS CID to an app-data hash.
#[wasm_bindgen(
    js_name = "cidToAppDataHex",
    unchecked_return_type = "WasmEnvelope<string>"
)]
pub fn cid_to_app_data_hex(cid: String) -> Result<JsValue, JsValue> {
    let hash = pure::app_data::cid_to_app_data_hex(&cid)
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(hash))
}

/// Returns the wasm crate version.
#[wasm_bindgen(js_name = "wasmVersion")]
#[must_use]
pub fn wasm_version() -> String {
    pure::chains::wasm_version()
}

pub use crate::exports::dto::OrderInput;
