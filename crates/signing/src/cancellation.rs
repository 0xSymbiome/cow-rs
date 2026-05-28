use std::fmt;

use cow_sdk_contracts::{OrderCancellations, SigningScheme};
use cow_sdk_core::{
    DigestSigner, OrderUid, ProtocolOptions, SignerError, SupportedChainId, TypedDataPayload,
    TypedDataSigner,
};

use crate::{
    SigningError,
    domain::{cancellation_fields, get_domain, serialize_message, typed_data_types},
    order_signing::{sign_with_scheme, signer_error},
};

/// Primary type name for `CoW` order-cancellation payloads.
pub const ORDER_CANCELLATIONS_PRIMARY_TYPE: &str = "OrderCancellations";

struct CancellationSigningPayload {
    payload: TypedDataPayload,
    digest: String,
}

/// Signs a single order cancellation using `Eip712`.
///
/// # Errors
///
/// Returns [`SigningError`] if payload construction, hashing, or signer execution fails.
pub async fn sign_order_cancellation<S>(
    order_uid: &OrderUid,
    chain_id: SupportedChainId,
    signer: &S,
    options: Option<&ProtocolOptions>,
) -> Result<crate::SigningResult, SigningError>
where
    S: TypedDataSigner,
    S::Error: fmt::Display + SignerError,
{
    sign_order_cancellations(std::slice::from_ref(order_uid), chain_id, signer, options).await
}

/// Signs a single order cancellation using an explicit local signing scheme.
///
/// # Errors
///
/// Returns [`SigningError`] if payload construction, hashing, or signer execution fails.
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        skip_all,
        fields(
            chain = ?chain_id,
            scheme = ?scheme,
            endpoint = "signing.order_cancellation",
        ),
    ),
)]
pub async fn sign_order_cancellation_with_scheme<S>(
    order_uid: &OrderUid,
    chain_id: SupportedChainId,
    signer: &S,
    scheme: SigningScheme,
    options: Option<&ProtocolOptions>,
) -> Result<crate::SigningResult, SigningError>
where
    S: TypedDataSigner + DigestSigner<Error = <S as TypedDataSigner>::Error>,
    <S as TypedDataSigner>::Error: fmt::Display + SignerError,
{
    sign_order_cancellations_with_scheme(
        std::slice::from_ref(order_uid),
        chain_id,
        signer,
        scheme,
        options,
    )
    .await
}

/// Signs a batch order cancellation using `Eip712`.
///
/// # Errors
///
/// Returns [`SigningError`] if payload construction, hashing, or signer execution fails.
pub async fn sign_order_cancellations<S>(
    order_uids: &[OrderUid],
    chain_id: SupportedChainId,
    signer: &S,
    options: Option<&ProtocolOptions>,
) -> Result<crate::SigningResult, SigningError>
where
    S: TypedDataSigner,
    S::Error: fmt::Display + SignerError,
{
    #[cfg(feature = "tracing")]
    tracing::debug!(
        target: "cow_sdk::signing",
        order_uid = %order_uids.first().map(OrderUid::to_hex_string).as_deref().unwrap_or("<empty>"),
        order_uid_count = order_uids.len(),
        "signing order cancellation",
    );
    let payload = cancellation_signing_payload(order_uids, chain_id, options)?;
    let raw = signer
        .sign_typed_data_payload(&payload.payload)
        .await
        .map_err(|error| signer_error("sign_typed_data_payload", error))?;
    Ok(crate::SigningResult {
        signature: cow_sdk_contracts::RecoverableSignature::parse_hex(&raw)?.to_hex_string(),
        signing_scheme: SigningScheme::Eip712,
    })
}

/// Signs a batch order cancellation using an explicit local signing scheme.
///
/// # Errors
///
/// Returns [`SigningError`] if payload construction, hashing, or signer execution fails.
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        skip_all,
        fields(
            chain = ?chain_id,
            scheme = ?scheme,
            endpoint = "signing.order_cancellations",
        ),
    ),
)]
pub async fn sign_order_cancellations_with_scheme<S>(
    order_uids: &[OrderUid],
    chain_id: SupportedChainId,
    signer: &S,
    scheme: SigningScheme,
    options: Option<&ProtocolOptions>,
) -> Result<crate::SigningResult, SigningError>
where
    S: TypedDataSigner + DigestSigner<Error = <S as TypedDataSigner>::Error>,
    <S as TypedDataSigner>::Error: fmt::Display + SignerError,
{
    #[cfg(feature = "tracing")]
    tracing::debug!(
        target: "cow_sdk::signing",
        order_uid = %order_uids.first().map(OrderUid::to_hex_string).as_deref().unwrap_or("<empty>"),
        order_uid_count = order_uids.len(),
        "signing order cancellation",
    );
    let payload = cancellation_signing_payload(order_uids, chain_id, options)?;
    sign_with_scheme(signer, scheme, &payload.payload, &payload.digest).await
}

/// Builds the signer-facing payload for a single order cancellation.
///
/// # Errors
///
/// Returns [`SigningError`] if domain construction or message serialization fails.
pub fn order_cancellation_typed_data_payload(
    order_uid: &OrderUid,
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<TypedDataPayload, SigningError> {
    order_cancellations_typed_data_payload(std::slice::from_ref(order_uid), chain_id, options)
}

/// Builds the signer-facing payload for a batch order cancellation.
///
/// # Errors
///
/// Returns [`SigningError`] if domain construction or message serialization fails.
pub fn order_cancellations_typed_data_payload(
    order_uids: &[OrderUid],
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<TypedDataPayload, SigningError> {
    let cancellations = OrderCancellations::new(order_uids.to_vec());

    Ok(TypedDataPayload::new(
        get_domain(chain_id, options)?,
        ORDER_CANCELLATIONS_PRIMARY_TYPE.to_owned(),
        typed_data_types(ORDER_CANCELLATIONS_PRIMARY_TYPE, cancellation_fields()),
        serialize_message(&cancellations)?,
    ))
}

fn cancellation_signing_payload(
    order_uids: &[OrderUid],
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<CancellationSigningPayload, SigningError> {
    let payload = order_cancellations_typed_data_payload(order_uids, chain_id, options)?;
    let cancellations = OrderCancellations::new(order_uids.to_vec());
    let digest = cow_sdk_contracts::hash_order_cancellations(&payload.domain, &cancellations)?;

    Ok(CancellationSigningPayload {
        payload,
        digest: digest.to_hex_string(),
    })
}
