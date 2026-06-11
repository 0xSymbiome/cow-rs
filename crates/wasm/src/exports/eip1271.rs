use crate::helpers as pure;
#[cfg(feature = "trading")]
use async_trait::async_trait;
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

/// Pure `Send + Sync` EIP-1271 signature provider over a host-resolved signature.
///
/// The JavaScript wallet adapter resolves the final contract signature at the
/// facade boundary; this provider hands that resolved value to the managed
/// submission path without retaining any JavaScript object, so the trading
/// workflow keeps a pure provider behind its `Arc<dyn …>` seam.
#[cfg(feature = "trading")]
pub(crate) struct ResolvedEip1271Provider {
    signature: String,
}

#[cfg(feature = "trading")]
impl ResolvedEip1271Provider {
    /// Wraps an already-resolved EIP-1271 contract signature.
    pub(crate) const fn new(signature: String) -> Self {
        Self { signature }
    }
}

#[cfg(feature = "trading")]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl cow_sdk_signing::eip1271::Eip1271Signer for ResolvedEip1271Provider {
    async fn sign(
        &self,
        _order_to_sign: &cow_sdk_core::OrderData,
    ) -> Result<String, cow_sdk_signing::eip1271::Eip1271SignatureError> {
        Ok(self.signature.clone())
    }
}

/// Compile-time guarantee that the resolved provider never captures a
/// non-`Send` JavaScript handle. It must stay `Send + Sync` (it holds only a
/// resolved signature string) so the trading workflow can keep it behind an
/// `Arc<dyn Eip1271Signer>`; this bound is enforced on every target
/// build, including `wasm32`, where `JsValue` would otherwise be `!Send`.
#[cfg(feature = "trading")]
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ResolvedEip1271Provider>();
};

/// Encodes a CoW EIP-1271 payload from an ECDSA order signature.
///
/// Use this pure helper when a smart-account flow already has the wrapped ECDSA
/// signature and needs the contract-signature payload bytes expected by CoW
/// Protocol order submission.
///
/// @param input Unsigned order used to derive the EIP-1271 payload.
/// @param ecdsaSignature Wrapped ECDSA signature as a `0x`-prefixed string.
/// @returns A versioned envelope containing the encoded EIP-1271 payload.
/// @throws CowError when the order or signature is invalid.
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
///
/// The SDK sends the EIP-712 envelope to the provided typed-data callback,
/// then converts the returned ECDSA signature into the CoW EIP-1271 payload.
/// Per-call options may attach cancellation and wallet timeout settings.
///
/// @param input Unsigned order to sign.
/// @param chainId EVM chain id for the EIP-712 domain.
/// @param owner Smart-account owner address used in the generated order UID.
/// @param typedDataSigner Callback that signs the typed-data envelope.
/// @param options Optional cancellation, timeout, and wallet timeout settings.
/// @returns A versioned envelope containing the signed-order DTO.
/// @throws CowError for invalid input, callback failure, timeout, or cancellation.
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
    super::traced("wasm.eip1271.sign_order_with_eip1271", async move {
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
            let signature = pure::signing::eip1271_signature_payload(&order, &ecdsa_signature)
                .map_err(|error| WasmError::from(error).into_js())?;
            let generated = pure::signing::generate_order_id(chain, &order, &owner)
                .map_err(|error| WasmError::from(error).into_js())?;
            let signed: SignedOrderDto =
                signed_order_from_parts(generated, owner, typed_data, signature, "eip1271", None);
            to_js_value(&WasmEnvelope::v1(signed))
        })
        .await
    })
    .await
}

/// Signs an order through a custom EIP-1271 callback.
///
/// Use this method when the JavaScript host owns the smart-account or
/// account-abstraction client and can return the final contract signature
/// directly. The SDK still builds typed data and the deterministic order UID.
///
/// @param input Unsigned order to sign.
/// @param chainId EVM chain id for the EIP-712 domain.
/// @param owner Smart-account owner address used in the generated order UID.
/// @param customCallback Callback that returns the final EIP-1271 signature.
/// @param options Optional cancellation, timeout, and wallet timeout settings.
/// @returns A versioned envelope containing the signed-order DTO.
/// @throws CowError for invalid input, callback failure, timeout, or cancellation.
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
    super::traced("wasm.eip1271.sign_order_with_custom_eip1271", async move {
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
    })
    .await
}
