use std::fmt;

use cow_sdk_contracts::{OrderCancellations, SigningScheme};
use cow_sdk_core::{AsyncSigner, OrderUid, ProtocolOptions, Signer, SupportedChainId};

use crate::{
    SigningError,
    domain::{cancellation_fields, get_domain},
    order_signing::{serialize, sign_with_scheme, sign_with_scheme_async},
};

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
    let domain = get_domain(chain_id, options)?;
    let cancellations = OrderCancellations {
        order_uids: order_uids.to_vec(),
    };
    let value_json = serialize(&cancellations)?;
    let digest = cow_sdk_contracts::hash_order_cancellations(&domain, &cancellations)?;
    sign_with_scheme(
        signer,
        scheme,
        &domain,
        &cancellation_fields(),
        &value_json,
        &digest,
    )
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
    let domain = get_domain(chain_id, options)?;
    let cancellations = OrderCancellations {
        order_uids: order_uids.to_vec(),
    };
    let value_json = serialize(&cancellations)?;
    let digest = cow_sdk_contracts::hash_order_cancellations(&domain, &cancellations)?;
    sign_with_scheme_async(
        signer,
        scheme,
        &domain,
        &cancellation_fields(),
        &value_json,
        &digest,
    )
    .await
}
