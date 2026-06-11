#[cfg(feature = "cancellation")]
use alloy_primitives::Bytes as AlloyBytes;
#[cfg(feature = "cancellation")]
use alloy_sol_types::SolCall as _;
use cow_sdk_contracts::RecoverableSignature;
#[cfg(feature = "cancellation")]
use cow_sdk_contracts::settlement::IGPv2Settlement;
#[cfg(feature = "cancellation")]
use cow_sdk_contracts::{ContractId, Registry};
use std::{cell::RefCell, rc::Rc};

use crate::helpers as pure;
use cow_sdk_core::{Address, DigestSigner, Eip1193};
#[cfg(feature = "cancellation")]
use cow_sdk_core::{Amount, Hash32, HexData, OrderUid, TransactionRequest};
use cow_sdk_signing::GeneratedOrderId;
#[cfg(feature = "cancellation")]
use cow_sdk_signing::order_cancellations_typed_data_payload;
use js_sys::{Array, Function, Promise, Reflect};
use serde_json::json;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::exports::{
    cancel::{ClientCallScope, SigningOptions, run_with_client_options, signing_wallet_timeout_ms},
    dto::{
        Eip1193Request, OrderInput, SignedOrderDto, TypedDataEnvelopeDto, parse_chain, parse_order,
        parse_owner, to_js_value, typed_data_json,
    },
    envelope::WasmEnvelope,
    errors::WasmError,
};

#[cfg(feature = "cancellation")]
use crate::exports::dto::{
    OrderTraderParametersInput, SignedCancellationsInput, TransactionRequestDto,
};

// The `cancellation` feature does not depend on `cow-sdk-trading`, so this
// settlement-tx gas fallback is kept local and in step with
// `cow_sdk_trading::DEFAULT_GAS_LIMIT` (mirrors upstream `@cowprotocol/cow-sdk`,
// which likewise keeps a separate cancellation gas default).
#[cfg(feature = "cancellation")]
const DEFAULT_GAS_LIMIT: u32 = 150_000;

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

impl DigestSigner for JsDigestSigner {
    // Carry the typed `JsValue` rather than flattening to `String`: a
    // `walletTimeout` raised inside `await_callback_string` must propagate with
    // its `kind` and `timeoutMs` intact instead of being collapsed to a message
    // and re-wrapped as a `walletRequest` at the call site. `DigestSigner::Error`
    // is an unconstrained associated type, so the adapter keeps the already-typed
    // JS error.
    type Error = JsValue;

    async fn sign_digest(&self, digest: &[u8]) -> Result<String, Self::Error> {
        let digest = alloy_primitives::hex::encode_prefixed(digest);
        let signature = await_callback_string(
            &self.callback,
            JsValue::from_str(&digest),
            "eth_sign",
            self.wallet_timeout_ms,
        )
        .await?;
        normalize_signature(&signature)
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

impl Eip1193 for JsEip1193Requester {
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
///
/// The SDK builds the EIP-712 typed-data envelope, passes it to the callback,
/// normalizes the returned ECDSA signature, and returns the signed-order DTO
/// with the canonical order UID and digest.
///
/// @param input Unsigned order fields to sign.
/// @param chainId EVM chain id used for the EIP-712 domain.
/// @param owner Owner address used in the generated order UID.
/// @param typedDataSigner Callback that signs the typed-data envelope.
/// @param options Optional cancellation, timeout, and wallet timeout settings.
/// @returns A versioned envelope containing the signed order.
/// @throws CowError for invalid input, callback failure, timeout, or cancellation.
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
    super::traced(
        "wasm.signing.sign_order_with_typed_data_signer",
        async move {
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
        },
    )
    .await
}

/// Signs an order through an EIP-1193 request callback.
///
/// The callback receives an `eth_signTypedData_v4` request object with owner
/// address and serialized typed data. This is the bridge for injected wallets
/// and wallet-client libraries that expose an EIP-1193-style request function.
///
/// @param input Unsigned order fields to sign.
/// @param chainId EVM chain id used for the EIP-712 domain.
/// @param owner Owner address used in the wallet request and order UID.
/// @param requestCallback Callback that executes the EIP-1193 request.
/// @param options Optional cancellation, timeout, and wallet timeout settings.
/// @returns A versioned envelope containing the signed order.
/// @throws CowError for invalid input, wallet failure, timeout, or cancellation.
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
    super::traced("wasm.signing.sign_order_with_eip1193", async move {
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
                    &[owner_address.to_hex_string(), typed_data_string],
                )
                .await?;
            let signature = normalize_signature(&signature)?;
            let signed =
                build_signed_order(input, chain, owner_address, typed_data, signature, "eip712")?;
            to_js_value(&WasmEnvelope::v1(signed))
        })
        .await
    })
    .await
}

/// Signs an order digest through an explicit `eth_sign` callback.
///
/// The SDK computes the canonical order digest, passes the digest as a
/// `0x`-prefixed string to the callback, normalizes the signature, and returns
/// an `ethsign` signed-order DTO.
///
/// @param input Unsigned order fields to sign.
/// @param chainId EVM chain id used for the digest.
/// @param owner Owner address used in the generated order UID.
/// @param digestSigner Callback that signs the digest string.
/// @param options Optional cancellation, timeout, and wallet timeout settings.
/// @returns A versioned envelope containing the signed order.
/// @throws CowError for invalid input, callback failure, timeout, or cancellation.
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
    super::traced("wasm.signing.sign_order_eth_sign_digest", async move {
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
            let digest = alloy_primitives::hex::decode(
                generated
                    .order_digest
                    .to_hex_string()
                    .trim_start_matches("0x"),
            )
            .map_err(|error| WasmError::invalid("digest", error.to_string()).into_js())?;
            let signer = JsDigestSigner::new(digest_signer, wallet_timeout_ms);
            // `sign_digest` already normalizes the signature and surfaces a typed
            // `walletTimeout` / `walletRequest` error, so the result propagates with
            // `?` without a lossy re-wrap or a redundant second normalization.
            let signature = signer.sign_digest(&digest).await?;
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
    })
    .await
}

/// Builds a settlement pre-sign transaction for an order UID.
///
/// The returned transaction request targets the Settlement contract and encodes
/// `setPreSignature(bytes,bool)` with the order UID and `true` flag. The host
/// wallet remains responsible for transaction submission.
///
/// @param params Order UID, chain, environment, and optional deployment override.
/// @returns A versioned envelope containing the transaction request DTO.
/// @throws CowError when the chain, deployment, or order UID is invalid.
#[cfg(feature = "cancellation")]
#[wasm_bindgen(
    js_name = "buildPresignTx",
    unchecked_return_type = "WasmEnvelope<TransactionRequestDto>"
)]
pub fn build_presign_tx(params: OrderTraderParametersInput) -> Result<JsValue, JsValue> {
    let calldata = encode_set_pre_signature_calldata(params.order_uid.as_str())?;
    let tx = settlement_transaction(params, calldata)?;
    to_js_value(&WasmEnvelope::v1(TransactionRequestDto::from(&tx)))
}

/// Builds a settlement cancellation transaction for an order UID.
///
/// The returned transaction request targets the Settlement contract and encodes
/// `invalidateOrder(bytes)`. The host wallet remains responsible for submitting
/// and observing the transaction.
///
/// @param params Order UID, chain, environment, and optional deployment override.
/// @returns A versioned envelope containing the transaction request DTO.
/// @throws CowError when the chain, deployment, or order UID is invalid.
#[cfg(feature = "cancellation")]
#[wasm_bindgen(
    js_name = "buildCancelOrderTx",
    unchecked_return_type = "WasmEnvelope<TransactionRequestDto>"
)]
pub fn build_cancel_order_tx(params: OrderTraderParametersInput) -> Result<JsValue, JsValue> {
    let calldata = encode_invalidate_order_calldata(params.order_uid.as_str())?;
    let tx = settlement_transaction(params, calldata)?;
    to_js_value(&WasmEnvelope::v1(TransactionRequestDto::from(&tx)))
}

/// Signs cancellation typed data through a typed-data callback.
///
/// The SDK builds the batch cancellation EIP-712 payload for the provided order
/// UIDs and asks the callback to sign it. The response can be submitted through
/// `OrderBookClient.cancelOrders`.
///
/// @param orderUids One or more full order UIDs to cancel.
/// @param chainId EVM chain id used for the cancellation domain.
/// @param typedDataSigner Callback that signs the typed-data envelope.
/// @param options Optional cancellation, timeout, and wallet timeout settings.
/// @returns A versioned envelope containing signed cancellations.
/// @throws CowError for empty input, invalid UID, callback failure, or timeout.
#[cfg(feature = "cancellation")]
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
    super::traced(
        "wasm.signing.sign_cancellation_with_typed_data_signer",
        async move {
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
        },
    )
    .await
}

/// Signs cancellation typed data through an EIP-1193 callback.
///
/// The callback receives an `eth_signTypedData_v4` request object. Use this
/// helper when an injected wallet or wallet client owns typed-data signing.
///
/// @param orderUids One or more full order UIDs to cancel.
/// @param chainId EVM chain id used for the cancellation domain.
/// @param owner Owner address included in the EIP-1193 request parameters.
/// @param requestCallback Callback that executes the EIP-1193 request.
/// @param options Optional cancellation, timeout, and wallet timeout settings.
/// @returns A versioned envelope containing signed cancellations.
/// @throws CowError for invalid input, wallet failure, timeout, or cancellation.
#[cfg(feature = "cancellation")]
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
    super::traced("wasm.signing.sign_cancellation_with_eip1193", async move {
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
                    &[owner.to_hex_string(), typed_data_string],
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
    })
    .await
}

/// Signs a cancellation digest through an explicit `eth_sign` callback.
///
/// The SDK computes the canonical cancellation digest for the provided UIDs and
/// passes it to the digest signer callback as a `0x`-prefixed string.
///
/// @param orderUids One or more full order UIDs to cancel.
/// @param chainId EVM chain id used for the cancellation digest.
/// @param digestSigner Callback that signs the digest string.
/// @param options Optional cancellation, timeout, and wallet timeout settings.
/// @returns A versioned envelope containing signed cancellations.
/// @throws CowError for empty input, invalid UID, callback failure, or timeout.
#[cfg(feature = "cancellation")]
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
    super::traced(
        "wasm.signing.sign_cancellation_eth_sign_digest",
        async move {
            let options = options.as_ref().map(AsRef::as_ref);
            let scope = ClientCallScope::new(options)?;
            let wallet_timeout_ms = signing_wallet_timeout_ms(options)?;
            run_with_client_options(scope, async move {
                let (uids, _payload, digest) = cancellation_payload(order_uids, chain_id)?;
                let digest_bytes =
                    alloy_primitives::hex::decode(digest.to_hex_string().trim_start_matches("0x"))
                        .map_err(|error| {
                            WasmError::invalid("digest", error.to_string()).into_js()
                        })?;
                let signer = JsDigestSigner::new(digest_signer, wallet_timeout_ms);
                let signature = signer.sign_digest(&digest_bytes).await?;
                to_js_value(&WasmEnvelope::v1(SignedCancellationsInput {
                    order_uids: uid_strings(&uids),
                    signature,
                    signing_scheme: "ethsign".to_owned(),
                }))
            })
            .await
        },
    )
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
    RecoverableSignature::parse_hex(raw_hex)
        .map(|sig| sig.to_hex_string())
        .map_err(|error| WasmError::from(error).into_js())
}

// Only the `trading` module consumes this (and the `js_message` helper it
// wraps), so gate both to that feature — otherwise the orderbook/signing
// flavours, which compile `signing` without `trading`, warn dead-code.
#[cfg(feature = "trading")]
pub(crate) fn js_error_to_string(value: JsValue) -> String {
    js_message(&value)
}

pub(crate) fn wallet_js_error(method: &'static str, error: JsValue) -> JsValue {
    // Per the redaction policy (ADR 0053), the provider-authored `message` and
    // `data` payloads can echo caller secrets or RPC tokens, so neither crosses
    // the boundary. Only the structured EIP-1193 / JSON-RPC `code` — a safe
    // machine signal — survives; the human message is SDK-authored guidance.
    let code = Reflect::get(&error, &JsValue::from_str("code"))
        .ok()
        .and_then(|code| code.as_f64())
        .map(|code| code as i64);
    WasmError::wallet_from_code(method, code).into_js()
}

#[cfg(feature = "trading")]
pub(crate) fn js_message(value: &JsValue) -> String {
    Reflect::get(value, &JsValue::from_str("message"))
        .ok()
        .and_then(|message| message.as_string())
        .or_else(|| value.as_string())
        .unwrap_or_else(|| "JavaScript callback failed".to_owned())
}

pub(crate) fn signed_order_from_parts(
    generated: GeneratedOrderId,
    owner: Address,
    typed_data: TypedDataEnvelopeDto,
    signature: String,
    signing_scheme: &str,
    quote_id: Option<i64>,
) -> SignedOrderDto {
    SignedOrderDto {
        order_uid: generated.order_id.to_hex_string(),
        signature,
        signing_scheme: signing_scheme.to_owned(),
        from: owner.to_hex_string(),
        order_digest: generated.order_digest.to_hex_string(),
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

#[cfg(feature = "cancellation")]
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

#[cfg(feature = "cancellation")]
fn settlement_transaction(
    params: OrderTraderParametersInput,
    calldata: Vec<u8>,
) -> Result<TransactionRequest, JsValue> {
    let chain_id = params
        .chain_id
        .ok_or_else(|| WasmError::invalid("chainId", "chainId is required").into_js())?;
    let chain = parse_chain(chain_id)?;
    let env = pure::chains::env_from_str(params.env.as_deref())
        .map_err(|error| WasmError::from(error).into_js())?;
    let settlement = params
        .settlement_contract_override
        .as_ref()
        .and_then(|overrides| overrides.get(&u64::from(chain_id)))
        .map(|address| {
            Address::new(address.clone()).map_err(|error| {
                WasmError::invalid("settlementContractOverride", error.to_string()).into_js()
            })
        })
        .transpose()?
        .or_else(|| Registry::default().address(ContractId::Settlement, chain, env))
        .ok_or_else(|| {
            WasmError::invalid(
                "chainId",
                "settlement deployment is not available for this chain and environment",
            )
            .into_js()
        })?;
    let data = alloy_primitives::hex::encode_prefixed(calldata);
    Ok(TransactionRequest::new(
        Some(settlement),
        Some(HexData::new(data).map_err(|error| WasmError::from(error).into_js())?),
        Some(Amount::ZERO),
        Some(default_gas_limit()?),
    ))
}

#[cfg(feature = "cancellation")]
fn default_gas_limit() -> Result<Amount, JsValue> {
    Amount::new(DEFAULT_GAS_LIMIT.to_string())
        .map_err(|error| WasmError::invalid("gasLimit", error.to_string()).into_js())
}

/// Parses the JS-supplied order UID hex string and returns the typed
/// 56-byte payload as an [`alloy_primitives::Bytes`] suitable for the
/// `IGPv2Settlement::*Call` Solidity `bytes` field. Routes through
/// `OrderUid::new` so malformed input surfaces as a typed `WasmError`
/// rather than the previous ad-hoc hex decode path.
#[cfg(feature = "cancellation")]
fn order_uid_bytes_from_str(uid: &str) -> Result<AlloyBytes, JsValue> {
    let order_uid =
        OrderUid::new(uid.to_owned()).map_err(|error| WasmError::from(error).into_js())?;
    Ok(AlloyBytes::from(order_uid.as_slice().to_vec()))
}

/// Encodes the `setPreSignature(bytes,bool)` calldata for the given order
/// UID through the workspace `alloy::sol!`-generated
/// `IGPv2Settlement::setPreSignatureCall` binding (ADR 0012). The selector,
/// the dynamic-bytes head/length words, and the bool word are emitted at
/// compile time through `SolCall::abi_encode`.
#[cfg(feature = "cancellation")]
fn encode_set_pre_signature_calldata(uid: &str) -> Result<Vec<u8>, JsValue> {
    let order_uid_bytes = order_uid_bytes_from_str(uid)?;
    Ok(IGPv2Settlement::setPreSignatureCall {
        orderUid: order_uid_bytes,
        signed: true,
    }
    .abi_encode())
}

/// Encodes the `invalidateOrder(bytes)` calldata for the given order UID
/// through the `IGPv2Settlement::invalidateOrderCall` binding.
#[cfg(feature = "cancellation")]
fn encode_invalidate_order_calldata(uid: &str) -> Result<Vec<u8>, JsValue> {
    let order_uid_bytes = order_uid_bytes_from_str(uid)?;
    Ok(IGPv2Settlement::invalidateOrderCall {
        orderUid: order_uid_bytes,
    }
    .abi_encode())
}

#[cfg(feature = "cancellation")]
fn uid_strings(uids: &[OrderUid]) -> Vec<String> {
    uids.iter().map(|uid| uid.to_hex_string()).collect()
}

fn wallet_timeout_promise(timeout_ms: u32) -> (Promise, gloo_timers::callback::Timeout) {
    let reject_holder: Rc<RefCell<Option<Function>>> = Rc::new(RefCell::new(None));
    let reject_setter = Rc::clone(&reject_holder);
    let promise = Promise::new(&mut |_resolve, reject| {
        *reject_setter.borrow_mut() = Some(reject);
    });
    let error = WasmError::wallet_timeout(timeout_ms).into_js();
    let timer = gloo_timers::callback::Timeout::new(timeout_ms, move || {
        if let Some(reject) = reject_holder.borrow_mut().take() {
            let _ = reject.call1(&JsValue::NULL, &error);
        }
    });
    (promise, timer)
}

fn is_wasm_error_kind(value: &JsValue, expected: &str) -> bool {
    Reflect::get(value, &JsValue::from_str("kind"))
        .ok()
        .and_then(|kind| kind.as_string())
        .is_some_and(|kind| kind == expected)
}
