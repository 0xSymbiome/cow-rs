use std::sync::Arc;

use async_trait::async_trait;
use cow_sdk_core::UnsignedOrder;
use cow_sdk_pure_helpers as pure;
use cow_sdk_trading::{Eip1271SignatureProvider, TradingError};
use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::exports::{
    cancel::{ClientCallScope, SigningOptions, run_with_client_options, signing_wallet_timeout_ms},
    dto::{
        CowEip1271SignRequest, OrderInput, SignedOrderDto, TypedDataEnvelopeDto, parse_chain,
        parse_order, parse_owner, to_js_value,
    },
    envelope::WasmEnvelope,
    errors::WasmError,
    signing::{await_callback_string, signed_order_from_parts},
};

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
    #[wasm_bindgen(js_name = options)] options: Option<SigningOptions>,
) -> Result<JsValue, JsValue> {
    let options = options.as_ref().map(AsRef::as_ref);
    let scope = ClientCallScope::new(options)?;
    let wallet_timeout_ms = signing_wallet_timeout_ms(options)?;
    run_with_client_options(scope, async move {
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
            wallet_timeout_ms,
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
    })
    .await
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
        let request = CowEip1271SignRequest {
            order: input,
            typed_data: typed_data.clone(),
            owner,
            chain_id,
        };
        let signature = await_callback_string(
            &custom_callback,
            to_js_value(&request)?,
            "eip1271",
            wallet_timeout_ms,
        )
        .await?;
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
    })
    .await
}
