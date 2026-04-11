use std::fmt;

use cow_sdk_contracts::{OrderCancellations, SigningScheme};
use cow_sdk_core::{
    AsyncSigner, OrderUid, ProtocolOptions, Signer, SupportedChainId, TypedDataPayload,
};

use crate::{
    SigningError,
    domain::{cancellation_fields, get_domain, serialize_message, typed_data_types},
    order_signing::{sign_with_scheme, sign_with_scheme_async},
};

pub const ORDER_CANCELLATIONS_PRIMARY_TYPE: &str = "OrderCancellations";

struct CancellationSigningPayload {
    payload: TypedDataPayload,
    digest: String,
}

pub fn sign_order_cancellation<S>(
    order_uid: &OrderUid,
    chain_id: SupportedChainId,
    signer: &S,
    options: Option<&ProtocolOptions>,
) -> Result<crate::SigningResult, SigningError>
where
    S: Signer,
    S::Error: fmt::Display,
{
    sign_order_cancellation_with_scheme(order_uid, chain_id, signer, SigningScheme::Eip712, options)
}

pub async fn sign_order_cancellation_async<S>(
    order_uid: &OrderUid,
    chain_id: SupportedChainId,
    signer: &S,
    options: Option<&ProtocolOptions>,
) -> Result<crate::SigningResult, SigningError>
where
    S: AsyncSigner,
    S::Error: fmt::Display,
{
    sign_order_cancellation_with_scheme_async(
        order_uid,
        chain_id,
        signer,
        SigningScheme::Eip712,
        options,
    )
    .await
}

pub fn sign_order_cancellation_with_scheme<S>(
    order_uid: &OrderUid,
    chain_id: SupportedChainId,
    signer: &S,
    scheme: SigningScheme,
    options: Option<&ProtocolOptions>,
) -> Result<crate::SigningResult, SigningError>
where
    S: Signer,
    S::Error: fmt::Display,
{
    sign_order_cancellations_with_scheme(
        std::slice::from_ref(order_uid),
        chain_id,
        signer,
        scheme,
        options,
    )
}

pub async fn sign_order_cancellation_with_scheme_async<S>(
    order_uid: &OrderUid,
    chain_id: SupportedChainId,
    signer: &S,
    scheme: SigningScheme,
    options: Option<&ProtocolOptions>,
) -> Result<crate::SigningResult, SigningError>
where
    S: AsyncSigner,
    S::Error: fmt::Display,
{
    sign_order_cancellations_with_scheme_async(
        std::slice::from_ref(order_uid),
        chain_id,
        signer,
        scheme,
        options,
    )
    .await
}

pub fn sign_order_cancellations<S>(
    order_uids: &[OrderUid],
    chain_id: SupportedChainId,
    signer: &S,
    options: Option<&ProtocolOptions>,
) -> Result<crate::SigningResult, SigningError>
where
    S: Signer,
    S::Error: fmt::Display,
{
    sign_order_cancellations_with_scheme(
        order_uids,
        chain_id,
        signer,
        SigningScheme::Eip712,
        options,
    )
}

pub async fn sign_order_cancellations_async<S>(
    order_uids: &[OrderUid],
    chain_id: SupportedChainId,
    signer: &S,
    options: Option<&ProtocolOptions>,
) -> Result<crate::SigningResult, SigningError>
where
    S: AsyncSigner,
    S::Error: fmt::Display,
{
    sign_order_cancellations_with_scheme_async(
        order_uids,
        chain_id,
        signer,
        SigningScheme::Eip712,
        options,
    )
    .await
}

pub fn sign_order_cancellations_with_scheme<S>(
    order_uids: &[OrderUid],
    chain_id: SupportedChainId,
    signer: &S,
    scheme: SigningScheme,
    options: Option<&ProtocolOptions>,
) -> Result<crate::SigningResult, SigningError>
where
    S: Signer,
    S::Error: fmt::Display,
{
    let payload = cancellation_signing_payload(order_uids, chain_id, options)?;
    sign_with_scheme(signer, scheme, &payload.payload, &payload.digest)
}

pub async fn sign_order_cancellations_with_scheme_async<S>(
    order_uids: &[OrderUid],
    chain_id: SupportedChainId,
    signer: &S,
    scheme: SigningScheme,
    options: Option<&ProtocolOptions>,
) -> Result<crate::SigningResult, SigningError>
where
    S: AsyncSigner,
    S::Error: fmt::Display,
{
    let payload = cancellation_signing_payload(order_uids, chain_id, options)?;
    sign_with_scheme_async(signer, scheme, &payload.payload, &payload.digest).await
}

pub fn order_cancellation_typed_data_payload(
    order_uid: &OrderUid,
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<TypedDataPayload, SigningError> {
    order_cancellations_typed_data_payload(std::slice::from_ref(order_uid), chain_id, options)
}

pub fn order_cancellations_typed_data_payload(
    order_uids: &[OrderUid],
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<TypedDataPayload, SigningError> {
    let cancellations = OrderCancellations {
        order_uids: order_uids.to_vec(),
    };

    Ok(TypedDataPayload {
        domain: get_domain(chain_id, options)?,
        primary_type: ORDER_CANCELLATIONS_PRIMARY_TYPE.to_owned(),
        types: typed_data_types(ORDER_CANCELLATIONS_PRIMARY_TYPE, cancellation_fields()),
        message: serialize_message(&cancellations)?,
    })
}

fn cancellation_signing_payload(
    order_uids: &[OrderUid],
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<CancellationSigningPayload, SigningError> {
    let payload = order_cancellations_typed_data_payload(order_uids, chain_id, options)?;
    let cancellations = OrderCancellations {
        order_uids: order_uids.to_vec(),
    };
    let digest = cow_sdk_contracts::hash_order_cancellations(&payload.domain, &cancellations)?;

    Ok(CancellationSigningPayload {
        payload,
        digest: digest.as_str().to_owned(),
    })
}
