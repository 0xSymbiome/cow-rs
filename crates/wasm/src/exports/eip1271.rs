use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
};

use async_trait::async_trait;
use cow_sdk_core::UnsignedOrder;
use cow_sdk_pure_helpers as pure;
use cow_sdk_trading::{Eip1271SignatureProvider, TradingError};
use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::exports::{
    dto::{
        CowEip1271SignRequest, OrderInput, SignedOrderDto, TypedDataEnvelopeDto, parse_chain,
        parse_order, parse_owner, to_js_value,
    },
    envelope::WasmEnvelope,
    errors::WasmError,
    signing::{await_callback_string, js_error_to_string, signed_order_from_parts},
};

const MAX_EIP1271_ALLOCATION_ATTEMPTS: u32 = 16;

thread_local! {
    static EIP1271_CALLBACKS: RefCell<HashMap<Eip1271CallbackId, Function>> =
        RefCell::new(HashMap::new());
}

static NEXT_EIP1271_CALLBACK_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Eip1271CallbackId(u32);

/// EIP-1271 provider that holds only a resolved ECDSA signature.
pub struct ResolvedEip1271Provider {
    signature: String,
}

impl ResolvedEip1271Provider {
    /// Creates a provider from a resolved ECDSA signature.
    #[must_use]
    pub fn new(signature: String) -> Self {
        Self { signature }
    }
}

#[async_trait(?Send)]
impl Eip1271SignatureProvider for ResolvedEip1271Provider {
    async fn sign(&self, order_to_sign: &UnsignedOrder) -> Result<String, TradingError> {
        pure::signing::eip1271_signature_payload(order_to_sign, &self.signature)
            .map_err(TradingError::from)
    }
}

pub(crate) struct RegisteredEip1271Provider {
    callback_id: Eip1271CallbackId,
    owner: String,
    chain_id: u32,
}

impl RegisteredEip1271Provider {
    pub(crate) const fn new(callback_id: Eip1271CallbackId, owner: String, chain_id: u32) -> Self {
        Self {
            callback_id,
            owner,
            chain_id,
        }
    }
}

#[async_trait(?Send)]
impl Eip1271SignatureProvider for RegisteredEip1271Provider {
    async fn sign(&self, order_to_sign: &UnsignedOrder) -> Result<String, TradingError> {
        let callback =
            lookup_eip1271_callback(self.callback_id).ok_or_else(|| TradingError::Signer {
                operation: "eip1271_callback",
                message: "EIP-1271 callback is no longer registered"
                    .to_owned()
                    .into(),
            })?;
        let chain =
            pure::chains::supported_chain(self.chain_id).map_err(|error| TradingError::Signer {
                operation: "eip1271_callback",
                message: error.to_string().into(),
            })?;
        let payload = pure::signing::order_typed_data_payload(chain, order_to_sign)
            .map_err(TradingError::from)?;
        let typed_data =
            TypedDataEnvelopeDto::from_payload(&payload).map_err(|error| TradingError::Signer {
                operation: "eip1271_callback",
                message: js_error_to_string(error.into_js()).into(),
            })?;
        let request = CowEip1271SignRequest {
            order: OrderInput::from(order_to_sign),
            typed_data,
            owner: self.owner.clone(),
            chain_id: self.chain_id,
        };
        let value = to_js_value(&request).map_err(|error| TradingError::Signer {
            operation: "eip1271_callback",
            message: js_error_to_string(error).into(),
        })?;
        await_callback_string(&callback, value, "eip1271")
            .await
            .map_err(|error| TradingError::Signer {
                operation: "eip1271_callback",
                message: js_error_to_string(error).into(),
            })
    }
}

pub(crate) struct Eip1271CallbackGuard {
    id: Eip1271CallbackId,
}

impl Eip1271CallbackGuard {
    pub(crate) fn register(callback: Function) -> Result<Self, JsValue> {
        let id = allocate_eip1271_callback_id()?;
        EIP1271_CALLBACKS.with(|cell| {
            cell.borrow_mut().insert(id, callback);
        });
        Ok(Self { id })
    }

    pub(crate) const fn id(&self) -> Eip1271CallbackId {
        self.id
    }
}

impl Drop for Eip1271CallbackGuard {
    fn drop(&mut self) {
        unregister_eip1271_callback(self.id);
    }
}

/// Encodes a CoW EIP-1271 payload from an ECDSA signature.
#[wasm_bindgen(
    js_name = "eip1271SignaturePayload",
    unchecked_return_type = "WasmEnvelope<string>"
)]
pub fn eip1271_signature_payload_export(
    input: OrderInput,
    #[wasm_bindgen(js_name = ecdsaSignature)] ecdsa_signature: String,
) -> Result<JsValue, JsValue> {
    let order = parse_order(input)?;
    let payload = pure::signing::eip1271_signature_payload(&order, &ecdsa_signature)
        .map_err(|error| WasmError::from(error).into_js())?;
    to_js_value(&WasmEnvelope::v1(payload))
}

/// Signs an order through typed-data ECDSA and wraps it as EIP-1271.
#[wasm_bindgen(
    js_name = "signOrderWithEip1271",
    unchecked_return_type = "WasmEnvelope<SignedOrderDto>"
)]
pub async fn sign_order_with_eip1271(
    input: OrderInput,
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
    owner: String,
    #[wasm_bindgen(js_name = typedDataSigner, unchecked_param_type = "TypedDataSignerCallback")]
    typed_data_signer: Function,
) -> Result<JsValue, JsValue> {
    let order = parse_order(input.clone())?;
    let chain = parse_chain(chain_id)?;
    let owner = parse_owner(&owner)?;
    let payload = pure::signing::order_typed_data_payload(chain, &order)
        .map_err(|error| WasmError::from(error).into_js())?;
    let typed_data = TypedDataEnvelopeDto::from_payload(&payload)?;
    let ecdsa_signature = crate::exports::signing::await_callback_string(
        &typed_data_signer,
        typed_data.callback_value()?,
        "signTypedData",
    )
    .await?;
    let provider = Arc::new(ResolvedEip1271Provider::new(ecdsa_signature));
    let signature = provider
        .sign(&order)
        .await
        .map_err(|error| WasmError::from(error).into_js())?;
    let generated = pure::signing::generate_order_id(chain, &order, &owner)
        .map_err(|error| WasmError::from(error).into_js())?;
    let signed: SignedOrderDto =
        signed_order_from_parts(generated, owner, typed_data, signature, "eip1271", None);
    to_js_value(&WasmEnvelope::v1(signed))
}

/// Signs an order through a custom EIP-1271 callback.
#[wasm_bindgen(
    js_name = "signOrderWithCustomEip1271",
    unchecked_return_type = "WasmEnvelope<SignedOrderDto>"
)]
pub async fn sign_order_with_custom_eip1271(
    input: OrderInput,
    #[wasm_bindgen(js_name = chainId)] chain_id: u32,
    owner: String,
    #[wasm_bindgen(js_name = customCallback, unchecked_param_type = "CustomEip1271Callback")]
    custom_callback: Function,
) -> Result<JsValue, JsValue> {
    let order = parse_order(input.clone())?;
    let chain = parse_chain(chain_id)?;
    let owner_address = parse_owner(&owner)?;
    let payload = pure::signing::order_typed_data_payload(chain, &order)
        .map_err(|error| WasmError::from(error).into_js())?;
    let typed_data = TypedDataEnvelopeDto::from_payload(&payload)?;
    let request = CowEip1271SignRequest {
        order: input,
        typed_data: typed_data.clone(),
        owner,
        chain_id,
    };
    let signature =
        await_callback_string(&custom_callback, to_js_value(&request)?, "eip1271").await?;
    let generated = pure::signing::generate_order_id(chain, &order, &owner_address)
        .map_err(|error| WasmError::from(error).into_js())?;
    let signed = signed_order_from_parts(
        generated,
        owner_address,
        typed_data,
        signature,
        "eip1271",
        None,
    );
    to_js_value(&WasmEnvelope::v1(signed))
}

fn lookup_eip1271_callback(id: Eip1271CallbackId) -> Option<Function> {
    EIP1271_CALLBACKS.with(|cell| cell.borrow().get(&id).cloned())
}

fn unregister_eip1271_callback(id: Eip1271CallbackId) {
    EIP1271_CALLBACKS.with(|cell| {
        cell.borrow_mut().remove(&id);
    });
}

fn allocate_eip1271_callback_id() -> Result<Eip1271CallbackId, JsValue> {
    for _ in 0..MAX_EIP1271_ALLOCATION_ATTEMPTS {
        let raw = NEXT_EIP1271_CALLBACK_ID.fetch_add(1, Ordering::Relaxed);
        if raw == 0 {
            continue;
        }
        let id = Eip1271CallbackId(raw);
        let collision = EIP1271_CALLBACKS.with(|cell| cell.borrow().contains_key(&id));
        if !collision {
            return Ok(id);
        }
    }

    Err(WasmError::internal("EIP-1271 callback handle space exhausted").into_js())
}
