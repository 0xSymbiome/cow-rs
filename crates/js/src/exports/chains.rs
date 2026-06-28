use wasm_bindgen::prelude::*;

use crate::helpers as pure;

use crate::dto::to_js_value;
use crate::exports::{envelope::WasmEnvelope, errors::JsResultExt};

#[cfg(feature = "signing")]
use crate::dto::{GeneratedOrderUid, parse_chain, parse_owner, payload_to_envelope};

#[cfg(feature = "app-data")]
use crate::dto::{AppDataDocument, AppDataInfo, AppDataParams, ValidationResult};

/// Computes the CoW Protocol EIP-712 domain separator for a supported chain.
///
/// Use this helper when a JavaScript host needs to compare the domain hash used
/// by the Rust SDK with another signing stack. The input is an EVM chain id,
/// not a CoW environment selector.
///
/// @param chainId EVM chain id supported by the deployment registry.
/// @returns The `0x`-prefixed 32-byte domain separator.
/// @throws CowError when the chain is not supported.
#[wasm_bindgen(js_name = "domainSeparator")]
pub fn domain_separator(
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
) -> Result<JsValue, JsValue> {
    let separator = pure::chains::domain_separator(chain_id).map_js()?;
    to_js_value(&WasmEnvelope::v1(separator))
}

/// Builds signer-facing EIP-712 typed data for an unsigned order.
///
/// The returned envelope contains the domain, type map, primary type, and
/// order message that wallet libraries expect for EIP-712 signing. It is
/// deterministic for the provided order and chain id.
///
/// @param order Unsigned order fields using the native order shape.
/// @param chainId EVM chain id used for the EIP-712 domain.
/// @returns A versioned envelope containing typed-data DTO fields.
/// @throws CowError when order parsing or chain validation fails.
#[cfg(feature = "signing")]
#[wasm_bindgen(
    js_name = "orderTypedData",
    unchecked_return_type = "WasmEnvelope<TypedDataEnvelope<Value>>"
)]
pub fn order_typed_data(
    order: cow_sdk_core::OrderData,
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
) -> Result<JsValue, JsValue> {
    let chain = parse_chain(chain_id)?;
    let payload = pure::signing::order_typed_data_payload(chain, &order).map_js()?;
    to_js_value(&WasmEnvelope::v1(payload_to_envelope(&payload)?))
}

/// Computes the canonical order UID and order digest for an unsigned order.
///
/// The UID combines the EIP-712 order digest, owner address, and validity
/// timestamp using the same packing rules as the native Rust SDK.
///
/// @param order Unsigned order fields to hash and pack.
/// @param chainId EVM chain id used for the EIP-712 domain.
/// @param owner Order owner address included in the UID suffix.
/// @returns A versioned envelope with `orderUid` and `orderDigest`.
/// @throws CowError when the order, owner, or chain id is invalid.
#[cfg(feature = "signing")]
#[wasm_bindgen(
    js_name = "computeOrderUid",
    unchecked_return_type = "WasmEnvelope<GeneratedOrderUid>"
)]
pub fn compute_order_uid(
    order: cow_sdk_core::OrderData,
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
    owner: String,
) -> Result<JsValue, JsValue> {
    let chain = parse_chain(chain_id)?;
    let owner = parse_owner(&owner)?;
    let generated = pure::signing::generate_order_id(chain, &order, &owner).map_js()?;
    let dto: GeneratedOrderUid = pure::dto::generated_order_uid_dto(&generated);
    to_js_value(&WasmEnvelope::v1(dto))
}

/// Returns the EVM chain ids supported by the SDK deployment registry.
///
/// This is a pure helper and does not perform network I/O. The returned list is
/// suitable for runtime validation, UI selection, or capability checks before a
/// client is constructed.
///
/// @returns A typed array of supported EVM chain ids.
#[wasm_bindgen(js_name = "supportedChainIds")]
#[must_use]
pub fn supported_chain_ids() -> Vec<u32> {
    pure::chains::supported_chain_ids()
}

/// Returns canonical CoW Protocol deployment addresses for a chain.
///
/// The optional environment selects production or staging deployment data. When
/// omitted, the helper uses the SDK default environment.
///
/// @param chainId EVM chain id to resolve.
/// @param env Optional CoW environment name, such as `prod` or `staging`.
/// @returns Settlement, VaultRelayer, EthFlow, and AllowListAuth addresses.
/// @throws CowError when the chain or environment is unsupported.
#[wasm_bindgen(
    js_name = "deploymentAddresses",
    unchecked_return_type = "WasmEnvelope<DeploymentAddresses>"
)]
pub fn deployment_addresses(
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
    env: Option<String>,
) -> Result<JsValue, JsValue> {
    let addresses = pure::chains::deployment_addresses(chain_id, env.as_deref()).map_js()?;
    to_js_value(&WasmEnvelope::v1(addresses))
}

/// Returns wrapped-native token metadata for a chain.
///
/// Use this to recognise a wrap pair in a swap UI — compare a selected token's
/// address against the returned address — or to display the wrapped-native
/// token. This is a pure lookup and performs no network I/O.
///
/// @param chainId EVM chain id to resolve.
/// @returns The wrapped-native token address, symbol, and decimals.
/// @throws CowError when the chain is not supported.
#[wasm_bindgen(
    js_name = "wrappedNativeToken",
    unchecked_return_type = "WasmEnvelope<WrappedNativeToken>"
)]
pub fn wrapped_native_token(
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
) -> Result<JsValue, JsValue> {
    let token = pure::chains::wrapped_native_token(chain_id).map_js()?;
    to_js_value(&WasmEnvelope::v1(token))
}

/// Builds app-data content and returns its deterministic hash and CID.
///
/// Use this when a JavaScript host wants the SDK to construct the canonical
/// document and expose the values needed for order submission and storage.
///
/// @param doc App-data document input accepted by the SDK schema.
/// @returns A versioned envelope containing document, hash, CID, and hex data.
/// @throws CowError when the document cannot be normalized or hashed.
#[cfg(feature = "app-data")]
#[wasm_bindgen(
    js_name = "appDataInfo",
    unchecked_return_type = "WasmEnvelope<AppDataInfo>"
)]
pub fn app_data_info(doc: AppDataParams) -> Result<JsValue, JsValue> {
    let document = pure::app_data::document_from_input(doc).map_js()?;
    let info: AppDataInfo = pure::app_data::app_data_info(&document).map_js()?;
    to_js_value(&WasmEnvelope::v1(info))
}

/// Builds app-data with the SDK's standard metadata and returns its hash and content.
///
/// This is the high-level counterpart to [`app_data_info`]: it stamps the quote
/// slippage, the given order class, and — unless a caller later overrides it via the
/// low-level path — the default SDK UTM attribution, exactly as the swap/limit flows
/// attach automatically. `orderClass` is the app-data order class (`market`, `limit`,
/// `liquidity`, or `twap`), distinct from the order-book order class.
///
/// @param appCode The dApp's app code.
/// @param slippageBps Slippage tolerance in basis points, recorded in `metadata.quote`.
/// @param orderClass App-data order class: `market`, `limit`, `liquidity`, or `twap`.
/// @returns A versioned envelope containing document, hash, CID, and hex data.
/// @throws CowError when the document cannot be built or hashed.
#[cfg(feature = "trading")]
#[wasm_bindgen(
    js_name = "buildAppData",
    unchecked_return_type = "WasmEnvelope<AppDataInfo>"
)]
pub fn build_app_data(
    #[wasm_bindgen(js_name = appCode)] app_code: String,
    #[wasm_bindgen(js_name = slippageBps)] slippage_bps: u32,
    #[wasm_bindgen(js_name = orderClass)] order_class: String,
) -> Result<JsValue, JsValue> {
    let code = cow_sdk_core::AppCode::new(app_code).map_js()?;
    let built = cow_sdk_trading::build_app_data_doc(&code, slippage_bps, &order_class, None, None)
        .map_js()?;
    let info: AppDataInfo = pure::app_data::app_data_info(&built.doc).map_js()?;
    to_js_value(&WasmEnvelope::v1(info))
}

/// Validates an app-data document against the typed metadata contract.
///
/// Validation is local and deterministic. The result reports whether the
/// document conforms and includes validation details without uploading data.
///
/// @param doc App-data document input to validate.
/// @returns A versioned envelope containing the validation result.
/// @throws CowError when the input cannot be converted into a document.
#[cfg(feature = "app-data")]
#[wasm_bindgen(
    js_name = "validateAppDataDoc",
    unchecked_return_type = "WasmEnvelope<ValidationResult>"
)]
pub fn validate_app_data_doc(doc: AppDataParams) -> Result<JsValue, JsValue> {
    let document = pure::app_data::document_from_input(doc).map_js()?;
    let result = pure::app_data::validate_app_data_doc(&document);
    to_js_value(&WasmEnvelope::v1(ValidationResult::from(result)))
}

/// Builds a normalized app-data document without deriving storage metadata.
///
/// This helper is useful when a host wants to inspect or modify the canonical
/// document shape before separately deriving app-data information.
///
/// @param doc App-data document input accepted by the SDK schema.
/// @returns A versioned envelope containing the normalized document.
/// @throws CowError when the input cannot be normalized.
#[cfg(feature = "app-data")]
#[wasm_bindgen(
    js_name = "appDataDoc",
    unchecked_return_type = "WasmEnvelope<AppDataDocument>"
)]
pub fn app_data_doc(doc: AppDataParams) -> Result<JsValue, JsValue> {
    let document = pure::app_data::document_from_input(doc).map_js()?;
    to_js_value(&WasmEnvelope::v1(AppDataDocument::from(document)))
}

/// Converts a `0x`-prefixed app-data hash into the canonical IPFS CID.
///
/// The conversion is pure and uses the same app-data multicodec and multihash
/// rules as the Rust app-data crate.
///
/// @param appDataHex App-data hash as a `0x`-prefixed hex string.
/// @returns A versioned envelope containing the CID string.
/// @throws CowError when the hash is malformed.
#[cfg(feature = "app-data")]
#[wasm_bindgen(
    js_name = "appDataHexToCid",
    unchecked_return_type = "WasmEnvelope<string>"
)]
pub fn app_data_hex_to_cid(
    #[wasm_bindgen(js_name = appDataHex)] app_data_hex: String,
) -> Result<JsValue, JsValue> {
    let cid = pure::app_data::app_data_hex_to_cid(&app_data_hex).map_js()?;
    to_js_value(&WasmEnvelope::v1(cid))
}

/// Converts a canonical IPFS CID into a `0x`-prefixed app-data hash.
///
/// Use this helper when an order or metadata path starts from a CID but the
/// orderbook request needs the app-data hash form.
///
/// @param cid Canonical CID string for an app-data document.
/// @returns A versioned envelope containing the `0x`-prefixed hash.
/// @throws CowError when the CID does not match the supported app-data shape.
#[cfg(feature = "app-data")]
#[wasm_bindgen(
    js_name = "cidToAppDataHex",
    unchecked_return_type = "WasmEnvelope<string>"
)]
pub fn cid_to_app_data_hex(cid: String) -> Result<JsValue, JsValue> {
    let hash = pure::app_data::cid_to_app_data_hex(&cid).map_js()?;
    to_js_value(&WasmEnvelope::v1(hash))
}

/// Returns the version of the wasm package runtime.
///
/// The value comes from the Rust package metadata used to build the wasm
/// artifact and can be included in diagnostics or compatibility checks.
///
/// @returns The semantic version string for this wasm build.
#[wasm_bindgen(js_name = "wasmVersion")]
#[must_use]
pub fn wasm_version() -> String {
    pure::chains::wasm_version()
}
