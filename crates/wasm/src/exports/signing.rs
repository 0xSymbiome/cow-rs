use cow_sdk_contracts::{ContractId, Registry, normalized_ecdsa_signature};
use std::{cell::RefCell, rc::Rc};

use cow_sdk_core::{
    Address, Amount, AsyncDigestSigner, AsyncEip1193, AsyncOwner, AsyncTypedDataSigner, Hash32,
    HexData, OrderUid, ProtocolOptions, TransactionRequest, TypedDataDomain, TypedDataField,
    TypedDataPayload,
};
use cow_sdk_pure_helpers as pure;
use cow_sdk_signing::order_cancellations_typed_data_payload;
use cow_sdk_trading::GAS_LIMIT_DEFAULT;
use js_sys::{Array, Function, Promise, Reflect};
use serde_json::json;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::JsFuture;

use crate::exports::{
    cancel::{ClientCallScope, SigningOptions, run_with_client_options, signing_wallet_timeout_ms},
    dto::{
        Eip1193Request, OrderInput, OrderTraderParametersInput, SignedCancellationsInput,
        SignedOrderDto, TransactionRequestDto, TypedDataEnvelopeDto, from_json_value, parse_chain,
        parse_order, parse_owner, to_js_value, typed_data_json,
    },
    envelope::WasmEnvelope,
    errors::WasmError,
};

/// Asynchronous typed-data signer backed by a JavaScript callback.
pub(crate) struct JsTypedDataSigner {
    owner: Address,
    callback: Function,
    wallet_timeout_ms: Option<u32>,
}

impl JsTypedDataSigner {
    pub(crate) const fn new(
        owner: Address,
        callback: Function,
        wallet_timeout_ms: Option<u32>,
    ) -> Self {
        Self {
            owner,
            callback,
            wallet_timeout_ms,
        }
    }
}

impl AsyncOwner for JsTypedDataSigner {
    type Error = String;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.owner.clone())
    }
}

impl AsyncTypedDataSigner for JsTypedDataSigner {
    type Error = String;

    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        let envelope = TypedDataEnvelopeDto::from_payload(payload)
            .map_err(|error| js_error_to_string(error.into_js()))?;
        let value = envelope.callback_value().map_err(js_error_to_string)?;
        let signature = await_callback_string(
            &self.callback,
            value,
            "signTypedData",
            self.wallet_timeout_ms,
        )
        .await
        .map_err(js_error_to_string)?;
        normalize_signature(&signature).map_err(js_error_to_string)
    }

    async fn sign_typed_data(
        &self,
        _domain: &TypedDataDomain,
        _fields: &[TypedDataField],
        _value_json: &str,
    ) -> Result<String, Self::Error> {
        Err("field-based typed-data signing is not available through this callback".to_owned())
    }
}

pub(crate) struct JsDigestSigner {
    callback: Function,
    wallet_timeout_ms: Option<u32>,
}

impl JsDigestSigner {
    const fn new(callback: Function, wallet_timeout_ms: Option<u32>) -> Self {
        Self {
            callback,
            wallet_timeout_ms,
        }
    }
}

impl AsyncDigestSigner for JsDigestSigner {
    type Error = String;

    async fn sign_digest(&self, digest: &[u8]) -> Result<String, Self::Error> {
        let digest = format!("0x{}", hex::encode(digest));
        let signature = await_callback_string(
            &self.callback,
            JsValue::from_str(&digest),
            "eth_sign",
            self.wallet_timeout_ms,
        )
        .await
        .map_err(js_error_to_string)?;
        normalize_signature(&signature).map_err(js_error_to_string)
    }
}

pub(crate) struct JsEip1193Requester {
    callback: Function,
    wallet_timeout_ms: Option<u32>,
}

impl JsEip1193Requester {
    const fn new(callback: Function, wallet_timeout_ms: Option<u32>) -> Self {
        Self {
            callback,
            wallet_timeout_ms,
        }
    }
}

impl AsyncEip1193 for JsEip1193Requester {
    type Error = JsValue;

    async fn request(&self, method: &str, params: &[String]) -> Result<String, Self::Error> {
        let request = Eip1193Request {
            method: method.to_owned(),
            params: Some(params.iter().map(|param| json!(param)).collect()),
        };
        let value = to_js_value(&request)?;
        await_callback_string(
            &self.callback,
            value,
            "eth_signTypedData_v4",
            self.wallet_timeout_ms,
        )
        .await
    }
}

/// Signs an order through a typed-data callback.
#[wasm_bindgen(
    js_name = "signOrderWithTypedDataSigner",
    unchecked_return_type = "WasmEnvelope<SignedOrderDto>"
)]
pub async fn sign_order_with_typed_data_signer(
    input: OrderInput,
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
    owner: String,
    #[wasm_bindgen(js_name = typedDataSigner, unchecked_param_type = "TypedDataSignerCallback")]
    typed_data_signer: Function,
    #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
) -> Result<JsValue, JsValue> {
    let options = options.as_ref().map(AsRef::as_ref);
    let scope = ClientCallScope::new(options)?;
    let wallet_timeout_ms = signing_wallet_timeout_ms(options)?;
    run_with_client_options(scope, async move {
        let signed = sign_order_with_callback(
            input,
            chain_id,
            owner,
            typed_data_signer,
            wallet_timeout_ms,
            "eip712",
        )
        .await?;
        to_js_value(&WasmEnvelope::v1(signed))
    })
    .await
}

/// Signs an order through an EIP-1193 request callback.
#[wasm_bindgen(
    js_name = "signOrderWithEip1193",
    unchecked_return_type = "WasmEnvelope<SignedOrderDto>"
)]
pub async fn sign_order_with_eip1193(
    input: OrderInput,
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
    owner: String,
    #[wasm_bindgen(js_name = requestCallback, unchecked_param_type = "Eip1193RequestCallback")]
    request_callback: Function,
    #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
) -> Result<JsValue, JsValue> {
    let options = options.as_ref().map(AsRef::as_ref);
    let scope = ClientCallScope::new(options)?;
    let wallet_timeout_ms = signing_wallet_timeout_ms(options)?;
    run_with_client_options(scope, async move {
        let order = parse_order(input.clone())?;
        let chain = parse_chain(chain_id)?;
        let owner_address = parse_owner(&owner)?;
        let payload = pure::signing::order_typed_data_payload(chain, &order)
            .map_err(|error| WasmError::from(error).into_js())?;
        let typed_data = TypedDataEnvelopeDto::from_payload(&payload)?;
        let typed_data_string = serde_json::to_string(&typed_data_json(&typed_data))
            .map_err(|error| WasmError::from(error).into_js())?;
        let requester = JsEip1193Requester::new(request_callback, wallet_timeout_ms);
        let signature = requester
            .request(
                "eth_signTypedData_v4",
                &[owner_address.as_str().to_owned(), typed_data_string],
            )
            .await?;
        let signature = normalize_signature(&signature)?;
        let signed =
            build_signed_order(input, chain, owner_address, typed_data, signature, "eip712")?;
        to_js_value(&WasmEnvelope::v1(signed))
    })
    .await
}

/// Signs an order digest through an explicit `eth_sign` callback.
#[wasm_bindgen(
    js_name = "signOrderEthSignDigest",
    unchecked_return_type = "WasmEnvelope<SignedOrderDto>"
)]
pub async fn sign_order_eth_sign_digest(
    input: OrderInput,
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
    owner: String,
    #[wasm_bindgen(js_name = digestSigner, unchecked_param_type = "DigestSignerCallback")]
    digest_signer: Function,
    #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
) -> Result<JsValue, JsValue> {
    let options = options.as_ref().map(AsRef::as_ref);
    let scope = ClientCallScope::new(options)?;
    let wallet_timeout_ms = signing_wallet_timeout_ms(options)?;
    run_with_client_options(scope, async move {
        let order = parse_order(input.clone())?;
        let chain = parse_chain(chain_id)?;
        let owner_address = parse_owner(&owner)?;
        let typed_data = TypedDataEnvelopeDto::from_payload(
            &pure::signing::order_typed_data_payload(chain, &order)
                .map_err(|error| WasmError::from(error).into_js())?,
        )?;
        let generated = pure::signing::generate_order_id(chain, &order, &owner_address)
            .map_err(|error| WasmError::from(error).into_js())?;
        let digest = hex::decode(generated.order_digest.as_str().trim_start_matches("0x"))
            .map_err(|error| WasmError::invalid("digest", error.to_string()).into_js())?;
        let signer = JsDigestSigner::new(digest_signer, wallet_timeout_ms);
        let signature = signer
            .sign_digest(&digest)
            .await
            .map_err(|error| WasmError::wallet("eth_sign", error).into_js())?;
        let signature = normalize_signature(&signature)?;
        let signed = signed_order_from_parts(
            generated,
            owner_address,
            typed_data,
            signature,
            "ethsign",
            None,
        );
        to_js_value(&WasmEnvelope::v1(signed))
    })
    .await
}

/// Builds a settlement pre-sign transaction for an order UID.
#[wasm_bindgen(
    js_name = "buildPresignTx",
    unchecked_return_type = "WasmEnvelope<TransactionRequestDto>"
)]
pub fn build_presign_tx(params: OrderTraderParametersInput) -> Result<JsValue, JsValue> {
    let params: cow_sdk_trading::OrderTraderParameters =
        from_json_value("params", params.into_value()?)?;
    let tx = order_uid_transaction(&params, "setPreSignature(bytes,bool)", true)?;
    to_js_value(&WasmEnvelope::v1(TransactionRequestDto::from(&tx)))
}

/// Builds a settlement cancellation transaction for an order UID.
#[wasm_bindgen(
    js_name = "buildCancelOrderTx",
    unchecked_return_type = "WasmEnvelope<TransactionRequestDto>"
)]
pub fn build_cancel_order_tx(params: OrderTraderParametersInput) -> Result<JsValue, JsValue> {
    let params: cow_sdk_trading::OrderTraderParameters =
        from_json_value("params", params.into_value()?)?;
    let tx = order_uid_transaction(&params, "invalidateOrder(bytes)", false)?;
    to_js_value(&WasmEnvelope::v1(TransactionRequestDto::from(&tx)))
}

/// Signs cancellation typed data through a typed-data callback.
#[wasm_bindgen(
    js_name = "signCancellationWithTypedDataSigner",
    unchecked_return_type = "WasmEnvelope<SignedCancellationsInput>"
)]
pub async fn sign_cancellation_with_typed_data_signer(
    #[wasm_bindgen(js_name = orderUids)] order_uids: Vec<String>,
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
    #[wasm_bindgen(js_name = typedDataSigner, unchecked_param_type = "TypedDataSignerCallback")]
    typed_data_signer: Function,
    #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
) -> Result<JsValue, JsValue> {
    let options = options.as_ref().map(AsRef::as_ref);
    let scope = ClientCallScope::new(options)?;
    let wallet_timeout_ms = signing_wallet_timeout_ms(options)?;
    run_with_client_options(scope, async move {
        let (uids, payload, _digest) = cancellation_payload(order_uids, chain_id)?;
        let envelope = TypedDataEnvelopeDto::from_payload(&payload)?;
        let signature = await_callback_string(
            &typed_data_signer,
            envelope.callback_value()?,
            "signTypedData",
            wallet_timeout_ms,
        )
        .await?;
        let signature = normalize_signature(&signature)?;
        to_js_value(&WasmEnvelope::v1(SignedCancellationsInput {
            order_uids: uid_strings(&uids),
            signature,
            signing_scheme: "eip712".to_owned(),
        }))
    })
    .await
}

/// Signs cancellation typed data through an EIP-1193 callback.
#[wasm_bindgen(
    js_name = "signCancellationWithEip1193",
    unchecked_return_type = "WasmEnvelope<SignedCancellationsInput>"
)]
pub async fn sign_cancellation_with_eip1193(
    #[wasm_bindgen(js_name = orderUids)] order_uids: Vec<String>,
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
    owner: String,
    #[wasm_bindgen(js_name = requestCallback, unchecked_param_type = "Eip1193RequestCallback")]
    request_callback: Function,
    #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
) -> Result<JsValue, JsValue> {
    let options = options.as_ref().map(AsRef::as_ref);
    let scope = ClientCallScope::new(options)?;
    let wallet_timeout_ms = signing_wallet_timeout_ms(options)?;
    run_with_client_options(scope, async move {
        let owner = parse_owner(&owner)?;
        let (uids, payload, _digest) = cancellation_payload(order_uids, chain_id)?;
        let envelope = TypedDataEnvelopeDto::from_payload(&payload)?;
        let typed_data_string = serde_json::to_string(&typed_data_json(&envelope))
            .map_err(|error| WasmError::from(error).into_js())?;
        let requester = JsEip1193Requester::new(request_callback, wallet_timeout_ms);
        let signature = requester
            .request(
                "eth_signTypedData_v4",
                &[owner.as_str().to_owned(), typed_data_string],
            )
            .await?;
        let signature = normalize_signature(&signature)?;
        to_js_value(&WasmEnvelope::v1(SignedCancellationsInput {
            order_uids: uid_strings(&uids),
            signature,
            signing_scheme: "eip712".to_owned(),
        }))
    })
    .await
}

/// Signs a cancellation digest through an explicit `eth_sign` callback.
#[wasm_bindgen(
    js_name = "signCancellationEthSignDigest",
    unchecked_return_type = "WasmEnvelope<SignedCancellationsInput>"
)]
pub async fn sign_cancellation_eth_sign_digest(
    #[wasm_bindgen(js_name = orderUids)] order_uids: Vec<String>,
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
    #[wasm_bindgen(js_name = digestSigner, unchecked_param_type = "DigestSignerCallback")]
    digest_signer: Function,
    #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
) -> Result<JsValue, JsValue> {
    let options = options.as_ref().map(AsRef::as_ref);
    let scope = ClientCallScope::new(options)?;
    let wallet_timeout_ms = signing_wallet_timeout_ms(options)?;
    run_with_client_options(scope, async move {
        let (uids, _payload, digest) = cancellation_payload(order_uids, chain_id)?;
        let digest_bytes = hex::decode(digest.as_str().trim_start_matches("0x"))
            .map_err(|error| WasmError::invalid("digest", error.to_string()).into_js())?;
        let signer = JsDigestSigner::new(digest_signer, wallet_timeout_ms);
        let signature = signer
            .sign_digest(&digest_bytes)
            .await
            .map_err(|error| WasmError::wallet("eth_sign", error).into_js())?;
        let signature = normalize_signature(&signature)?;
        to_js_value(&WasmEnvelope::v1(SignedCancellationsInput {
            order_uids: uid_strings(&uids),
            signature,
            signing_scheme: "ethsign".to_owned(),
        }))
    })
    .await
}

pub(crate) async fn await_callback_string(
    callback: &Function,
    arg: JsValue,
    method: &'static str,
    wallet_timeout_ms: Option<u32>,
) -> Result<String, JsValue> {
    let value = callback
        .call1(&JsValue::NULL, &arg)
        .map_err(|error| wallet_js_error(method, error))?;
    let callback_promise = Promise::resolve(&value);
    let mut timeout_guard = None;
    let promise = if let Some(timeout_ms) = wallet_timeout_ms {
        let (timeout_promise, guard) = wallet_timeout_promise(timeout_ms);
        timeout_guard = Some(guard);
        let race = Array::new();
        race.push(&callback_promise);
        race.push(&timeout_promise);
        Promise::race(&race)
    } else {
        callback_promise
    };
    let awaited = JsFuture::from(promise).await;
    drop(timeout_guard);
    let value = awaited.map_err(|error| {
        if is_wasm_error_kind(&error, "walletTimeout") {
            error
        } else {
            wallet_js_error(method, error)
        }
    })?;
    value
        .as_string()
        .ok_or_else(|| WasmError::wallet(method, "callback did not return a string").into_js())
}

pub(crate) fn normalize_signature(raw_hex: &str) -> Result<String, JsValue> {
    normalized_ecdsa_signature(raw_hex).map_err(|error| WasmError::from(error).into_js())
}

pub(crate) fn js_error_to_string(value: JsValue) -> String {
    js_message(&value)
}

pub(crate) fn wallet_js_error(method: &'static str, error: JsValue) -> JsValue {
    let message = js_message(&error);
    let code = Reflect::get(&error, &JsValue::from_str("code"))
        .ok()
        .and_then(|code| code.as_f64())
        .map(|code| code as i64);
    let data = Reflect::get(&error, &JsValue::from_str("data"))
        .ok()
        .and_then(|data| serde_wasm_bindgen::from_value(data).ok());
    WasmError::WalletRequest {
        schema_version: crate::exports::SchemaVersion::V1,
        method: method.to_owned(),
        code,
        message,
        data,
    }
    .into_js()
}

pub(crate) fn js_message(value: &JsValue) -> String {
    Reflect::get(value, &JsValue::from_str("message"))
        .ok()
        .and_then(|message| message.as_string())
        .or_else(|| value.as_string())
        .unwrap_or_else(|| "JavaScript callback failed".to_owned())
}

pub(crate) fn signed_order_from_parts(
    generated: cow_sdk_signing::GeneratedOrderId,
    owner: Address,
    typed_data: TypedDataEnvelopeDto,
    signature: String,
    signing_scheme: &str,
    quote_id: Option<i64>,
) -> SignedOrderDto {
    SignedOrderDto {
        order_uid: generated.order_id.as_str().to_owned(),
        signature,
        signing_scheme: signing_scheme.to_owned(),
        from: owner.as_str().to_owned(),
        order_digest: generated.order_digest.as_str().to_owned(),
        typed_data,
        quote_id,
    }
}

async fn sign_order_with_callback(
    input: OrderInput,
    chain_id: u32,
    owner: String,
    typed_data_signer: Function,
    wallet_timeout_ms: Option<u32>,
    scheme: &str,
) -> Result<SignedOrderDto, JsValue> {
    let order = parse_order(input.clone())?;
    let chain = parse_chain(chain_id)?;
    let owner = parse_owner(&owner)?;
    let payload = pure::signing::order_typed_data_payload(chain, &order)
        .map_err(|error| WasmError::from(error).into_js())?;
    let typed_data = TypedDataEnvelopeDto::from_payload(&payload)?;
    let signature = await_callback_string(
        &typed_data_signer,
        typed_data.callback_value()?,
        "signTypedData",
        wallet_timeout_ms,
    )
    .await?;
    let signature = normalize_signature(&signature)?;
    build_signed_order(input, chain, owner, typed_data, signature, scheme)
}

fn build_signed_order(
    input: OrderInput,
    chain: cow_sdk_core::SupportedChainId,
    owner: Address,
    typed_data: TypedDataEnvelopeDto,
    signature: String,
    scheme: &str,
) -> Result<SignedOrderDto, JsValue> {
    let order = parse_order(input)?;
    let generated = pure::signing::generate_order_id(chain, &order, &owner)
        .map_err(|error| WasmError::from(error).into_js())?;
    Ok(signed_order_from_parts(
        generated, owner, typed_data, signature, scheme, None,
    ))
}

fn cancellation_payload(
    order_uids: Vec<String>,
    chain_id: u32,
) -> Result<(Vec<OrderUid>, cow_sdk_core::TypedDataPayload, Hash32), JsValue> {
    if order_uids.is_empty() {
        return Err(
            WasmError::invalid("orderUids", "at least one order UID is required").into_js(),
        );
    }

    let chain = parse_chain(chain_id)?;
    let uids = order_uids
        .into_iter()
        .map(|uid| {
            OrderUid::new(uid).map_err(|error| WasmError::invalid("orderUid", error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(WasmError::into_js)?;
    let payload = order_cancellations_typed_data_payload(&uids, chain, None)
        .map_err(|error| WasmError::from(error).into_js())?;
    let cancellations = cow_sdk_contracts::OrderCancellations::new(uids.clone());
    let digest = cow_sdk_contracts::hash_order_cancellations(&payload.domain, &cancellations)
        .map_err(|error| WasmError::from(error).into_js())?;
    Ok((uids, payload, digest))
}

fn order_uid_transaction(
    params: &cow_sdk_trading::OrderTraderParameters,
    selector: &'static str,
    include_bool: bool,
) -> Result<TransactionRequest, JsValue> {
    let chain_id = params
        .chain_id
        .ok_or_else(|| WasmError::invalid("chainId", "chainId is required").into_js())?;
    let mut options = ProtocolOptions::new();
    if let Some(env) = params.env {
        options = options.with_env(env);
    }
    if let Some(overrides) = params.settlement_contract_override.clone() {
        options = options.with_settlement_contract_override(overrides);
    }
    if let Some(overrides) = params.eth_flow_contract_override.clone() {
        options = options.with_eth_flow_contract_override(overrides);
    }
    let env = options.env.unwrap_or(cow_sdk_core::CowEnv::Prod);
    let settlement = options
        .settlement_contract_override
        .as_ref()
        .and_then(|overrides| overrides.get(&u64::from(chain_id)).cloned())
        .or_else(|| Registry::default().address(ContractId::Settlement, chain_id, env))
        .ok_or_else(|| {
            WasmError::invalid(
                "chainId",
                "settlement deployment is not available for this chain and environment",
            )
            .into_js()
        })?;
    let data = if include_bool {
        encode_selector_and_dynamic_bytes_bool(selector, params.order_uid.as_str(), true)?
    } else {
        encode_selector_and_dynamic_bytes(selector, params.order_uid.as_str())?
    };
    let tx = TransactionRequest::new(
        Some(settlement),
        Some(HexData::new(data).map_err(|error| WasmError::from(error).into_js())?),
        Some(Amount::zero()),
        Some(default_gas_limit()?),
    );
    Ok(tx)
}

fn default_gas_limit() -> Result<Amount, JsValue> {
    Amount::new(GAS_LIMIT_DEFAULT.to_string())
        .map_err(|error| WasmError::invalid("gasLimit", error.to_string()).into_js())
}

fn encode_selector_and_dynamic_bytes(signature: &str, bytes_hex: &str) -> Result<String, JsValue> {
    let selector = selector_bytes(signature)?;
    let bytes = decode_hex_field("bytes", bytes_hex)?;
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&selector);
    encoded.extend_from_slice(&encode_usize_word(32));
    encoded.extend_from_slice(&encode_usize_word(bytes.len()));
    encoded.extend_from_slice(&pad_to_word(bytes));
    Ok(format!("0x{}", hex::encode(encoded)))
}

fn encode_selector_and_dynamic_bytes_bool(
    signature: &str,
    bytes_hex: &str,
    flag: bool,
) -> Result<String, JsValue> {
    let selector = selector_bytes(signature)?;
    let bytes = decode_hex_field("bytes", bytes_hex)?;
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&selector);
    encoded.extend_from_slice(&encode_usize_word(64));
    encoded.extend_from_slice(&encode_bool_word(flag));
    encoded.extend_from_slice(&encode_usize_word(bytes.len()));
    encoded.extend_from_slice(&pad_to_word(bytes));
    Ok(format!("0x{}", hex::encode(encoded)))
}

fn selector_bytes(signature: &str) -> Result<[u8; 4], JsValue> {
    let selector = cow_sdk_contracts::function_magic_value(signature);
    let bytes = decode_hex_field("selector", &selector)?;
    let mut out = [0u8; 4];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn decode_hex_field(field: &'static str, value: &str) -> Result<Vec<u8>, JsValue> {
    let Some(stripped) = value.strip_prefix("0x") else {
        return Err(WasmError::invalid(field, "hex value must start with 0x").into_js());
    };
    hex::decode(stripped).map_err(|error| WasmError::invalid(field, error.to_string()).into_js())
}

fn encode_usize_word(value: usize) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[24..].copy_from_slice(&(value as u64).to_be_bytes());
    out
}

fn encode_bool_word(value: bool) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[31] = u8::from(value);
    out
}

fn pad_to_word(mut bytes: Vec<u8>) -> Vec<u8> {
    let padding = (32 - (bytes.len() % 32)) % 32;
    bytes.extend(std::iter::repeat_n(0u8, padding));
    bytes
}

fn uid_strings(uids: &[OrderUid]) -> Vec<String> {
    uids.iter().map(|uid| uid.as_str().to_owned()).collect()
}

struct WalletTimeoutGuard {
    parts: Rc<RefCell<Option<WalletTimeoutParts>>>,
}

struct WalletTimeoutParts {
    handle: JsValue,
    on_timeout: Closure<dyn FnMut()>,
}

impl Drop for WalletTimeoutGuard {
    fn drop(&mut self) {
        if let Some(parts) = self.parts.borrow_mut().take() {
            global_clear_timeout_raw(&parts.handle);
            drop(parts.on_timeout);
        }
    }
}

fn wallet_timeout_promise(timeout_ms: u32) -> (Promise, WalletTimeoutGuard) {
    let parts = Rc::new(RefCell::new(None));
    let parts_for_executor = Rc::clone(&parts);
    let promise = Promise::new(&mut |_resolve, reject| {
        let error = WasmError::wallet_timeout(timeout_ms).into_js();
        let on_timeout = Closure::<dyn FnMut()>::new(move || {
            let _ = reject.call1(&JsValue::NULL, &error);
        });
        let handle = global_set_timeout_raw(on_timeout.as_ref().unchecked_ref(), timeout_ms);
        *parts_for_executor.borrow_mut() = Some(WalletTimeoutParts { handle, on_timeout });
    });
    (promise, WalletTimeoutGuard { parts })
}

fn is_wasm_error_kind(value: &JsValue, expected: &str) -> bool {
    Reflect::get(value, &JsValue::from_str("kind"))
        .ok()
        .and_then(|kind| kind.as_string())
        .is_some_and(|kind| kind == expected)
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = globalThis, js_name = setTimeout)]
    fn global_set_timeout_raw(handler: &Function, ms: u32) -> JsValue;

    #[wasm_bindgen(js_namespace = globalThis, js_name = clearTimeout)]
    fn global_clear_timeout_raw(handle: &JsValue);
}
