use cow_sdk_contracts::normalized_ecdsa_signature;
use cow_sdk_core::{Address, Amount, AsyncSigner, Hash32, OrderUid, TransactionBroadcast};
use cow_sdk_signing::order_cancellations_typed_data_payload;
use js_sys::{Function, Promise, Reflect};
use serde_json::json;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::{
    exports::{
        dto::{
            Eip1193Request, OrderInput, SignedCancellationsInput, SignedOrderDto,
            TypedDataEnvelopeDto, parse_chain, parse_order, parse_owner, to_js_value,
            typed_data_json,
        },
        errors::WasmError,
    },
    pure,
};

/// Asynchronous typed-data signer backed by a JavaScript callback.
pub(crate) struct JsTypedDataSigner {
    owner: Address,
    callback: Function,
}

impl JsTypedDataSigner {
    pub(crate) const fn new(owner: Address, callback: Function) -> Self {
        Self { owner, callback }
    }
}

impl AsyncSigner for JsTypedDataSigner {
    type Error = String;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.owner.clone())
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Err("message signing is not available through this callback".to_owned())
    }

    async fn sign_transaction(
        &self,
        _tx: &cow_sdk_core::TransactionRequest,
    ) -> Result<String, Self::Error> {
        Err("transaction signing is not available through this callback".to_owned())
    }

    async fn sign_typed_data_payload(
        &self,
        payload: &cow_sdk_core::TypedDataPayload,
    ) -> Result<String, Self::Error> {
        let envelope = TypedDataEnvelopeDto::from_payload(payload)
            .map_err(|error| js_error_to_string(error.into_js()))?;
        let value = envelope.callback_value().map_err(js_error_to_string)?;
        let signature = await_callback_string(&self.callback, value, "signTypedData")
            .await
            .map_err(js_error_to_string)?;
        normalize_signature(&signature).map_err(js_error_to_string)
    }

    async fn sign_typed_data(
        &self,
        _domain: &cow_sdk_core::TypedDataDomain,
        _fields: &[cow_sdk_core::TypedDataField],
        _value_json: &str,
    ) -> Result<String, Self::Error> {
        Err("field-based typed-data signing is not available through this callback".to_owned())
    }

    async fn send_transaction(
        &self,
        _tx: &cow_sdk_core::TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Err("transaction broadcast is not available through this callback".to_owned())
    }

    async fn estimate_gas(
        &self,
        _tx: &cow_sdk_core::TransactionRequest,
    ) -> Result<Amount, Self::Error> {
        Err("gas estimation is not available through this callback".to_owned())
    }
}

/// Owner-only signer used when a flow signs through a separate callback provider.
pub(crate) struct OwnerOnlySigner {
    owner: Address,
}

impl OwnerOnlySigner {
    pub(crate) const fn new(owner: Address) -> Self {
        Self { owner }
    }
}

impl AsyncSigner for OwnerOnlySigner {
    type Error = String;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.owner.clone())
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Err("message signing is not available on the owner-only signer".to_owned())
    }

    async fn sign_transaction(
        &self,
        _tx: &cow_sdk_core::TransactionRequest,
    ) -> Result<String, Self::Error> {
        Err("transaction signing is not available on the owner-only signer".to_owned())
    }

    async fn sign_typed_data(
        &self,
        _domain: &cow_sdk_core::TypedDataDomain,
        _fields: &[cow_sdk_core::TypedDataField],
        _value_json: &str,
    ) -> Result<String, Self::Error> {
        Err("typed-data signing is not available on the owner-only signer".to_owned())
    }

    async fn send_transaction(
        &self,
        _tx: &cow_sdk_core::TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Err("transaction broadcast is not available on the owner-only signer".to_owned())
    }

    async fn estimate_gas(
        &self,
        _tx: &cow_sdk_core::TransactionRequest,
    ) -> Result<Amount, Self::Error> {
        Err("gas estimation is not available on the owner-only signer".to_owned())
    }
}

/// Signs an order through a typed-data callback.
#[wasm_bindgen(js_name = "signOrderWithTypedDataSigner")]
pub async fn sign_order_with_typed_data_signer(
    input: OrderInput,
    chain_id: u32,
    owner: String,
    typed_data_signer: Function,
) -> Result<JsValue, JsValue> {
    let signed =
        sign_order_with_callback(input, chain_id, owner, typed_data_signer, "eip712").await?;
    to_js_value(&signed)
}

/// Signs an order through an EIP-1193 request callback.
#[wasm_bindgen(js_name = "signOrderWithEip1193")]
pub async fn sign_order_with_eip1193(
    input: OrderInput,
    chain_id: u32,
    owner: String,
    request_callback: Function,
) -> Result<JsValue, JsValue> {
    let order = parse_order(input.clone())?;
    let chain = parse_chain(chain_id)?;
    let owner_address = parse_owner(&owner)?;
    let payload = pure::signing::order_typed_data_payload(chain, &order)
        .map_err(|error| WasmError::from(error).into_js())?;
    let typed_data = TypedDataEnvelopeDto::from_payload(&payload)?;
    let typed_data_string = serde_json::to_string(&typed_data_json(&typed_data))
        .map_err(|error| WasmError::from(error).into_js())?;
    let request = Eip1193Request {
        method: "eth_signTypedData_v4".to_owned(),
        params: Some(vec![
            json!(owner_address.as_str()),
            json!(typed_data_string),
        ]),
    };
    let value = to_js_value(&request)?;
    let signature = await_callback_string(&request_callback, value, "eth_signTypedData_v4").await?;
    let signature = normalize_signature(&signature)?;
    let signed = build_signed_order(input, chain, owner_address, typed_data, signature, "eip712")?;
    to_js_value(&signed)
}

/// Signs an order digest through an explicit `eth_sign` callback.
#[wasm_bindgen(js_name = "signOrderEthSignDigest")]
pub async fn sign_order_eth_sign_digest(
    input: OrderInput,
    chain_id: u32,
    owner: String,
    digest_signer: Function,
) -> Result<JsValue, JsValue> {
    let order = parse_order(input.clone())?;
    let chain = parse_chain(chain_id)?;
    let owner_address = parse_owner(&owner)?;
    let typed_data = TypedDataEnvelopeDto::from_payload(
        &pure::signing::order_typed_data_payload(chain, &order)
            .map_err(|error| WasmError::from(error).into_js())?,
    )?;
    let generated = pure::signing::generate_order_id(chain, &order, &owner_address)
        .map_err(|error| WasmError::from(error).into_js())?;
    let digest = generated.order_digest.as_str().to_owned();
    let signature =
        await_callback_string(&digest_signer, JsValue::from_str(&digest), "eth_sign").await?;
    let signature = normalize_signature(&signature)?;
    let signed = signed_order_from_parts(
        generated,
        owner_address,
        typed_data,
        signature,
        "ethsign",
        None,
    );
    to_js_value(&signed)
}

/// Signs cancellation typed data through a typed-data callback.
#[wasm_bindgen(js_name = "signCancellationWithTypedDataSigner")]
pub async fn sign_cancellation_with_typed_data_signer(
    order_uids: Vec<String>,
    chain_id: u32,
    typed_data_signer: Function,
) -> Result<JsValue, JsValue> {
    let (uids, payload, _digest) = cancellation_payload(order_uids, chain_id)?;
    let envelope = TypedDataEnvelopeDto::from_payload(&payload)?;
    let signature = await_callback_string(
        &typed_data_signer,
        envelope.callback_value()?,
        "signTypedData",
    )
    .await?;
    let signature = normalize_signature(&signature)?;
    to_js_value(&SignedCancellationsInput {
        order_uids: uid_strings(&uids),
        signature,
        signing_scheme: "eip712".to_owned(),
    })
}

/// Signs cancellation typed data through an EIP-1193 callback.
#[wasm_bindgen(js_name = "signCancellationWithEip1193")]
pub async fn sign_cancellation_with_eip1193(
    order_uids: Vec<String>,
    chain_id: u32,
    owner: String,
    request_callback: Function,
) -> Result<JsValue, JsValue> {
    let owner = parse_owner(&owner)?;
    let (uids, payload, _digest) = cancellation_payload(order_uids, chain_id)?;
    let envelope = TypedDataEnvelopeDto::from_payload(&payload)?;
    let typed_data_string = serde_json::to_string(&typed_data_json(&envelope))
        .map_err(|error| WasmError::from(error).into_js())?;
    let request = Eip1193Request {
        method: "eth_signTypedData_v4".to_owned(),
        params: Some(vec![json!(owner.as_str()), json!(typed_data_string)]),
    };
    let signature = await_callback_string(
        &request_callback,
        to_js_value(&request)?,
        "eth_signTypedData_v4",
    )
    .await?;
    let signature = normalize_signature(&signature)?;
    to_js_value(&SignedCancellationsInput {
        order_uids: uid_strings(&uids),
        signature,
        signing_scheme: "eip712".to_owned(),
    })
}

/// Signs a cancellation digest through an explicit `eth_sign` callback.
#[wasm_bindgen(js_name = "signCancellationEthSignDigest")]
pub async fn sign_cancellation_eth_sign_digest(
    order_uids: Vec<String>,
    chain_id: u32,
    digest_signer: Function,
) -> Result<JsValue, JsValue> {
    let (uids, _payload, digest) = cancellation_payload(order_uids, chain_id)?;
    let signature = await_callback_string(
        &digest_signer,
        JsValue::from_str(digest.as_str()),
        "eth_sign",
    )
    .await?;
    let signature = normalize_signature(&signature)?;
    to_js_value(&SignedCancellationsInput {
        order_uids: uid_strings(&uids),
        signature,
        signing_scheme: "ethsign".to_owned(),
    })
}

pub(crate) async fn await_callback_string(
    callback: &Function,
    arg: JsValue,
    method: &'static str,
) -> Result<String, JsValue> {
    let value = callback
        .call1(&JsValue::NULL, &arg)
        .map_err(|error| wallet_js_error(method, error))?;
    let promise = Promise::resolve(&value);
    let value = JsFuture::from(promise)
        .await
        .map_err(|error| wallet_js_error(method, error))?;
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
        schema_version: crate::exports::dto::SchemaVersion::V1,
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

fn uid_strings(uids: &[OrderUid]) -> Vec<String> {
    uids.iter().map(|uid| uid.as_str().to_owned()).collect()
}
