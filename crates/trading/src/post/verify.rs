use cow_sdk_core::{Provider, ProtocolOptions};

use crate::TradingError;

/// Builds an EIP-1271 verification request for a `CoW` order digest.
///
/// # Errors
///
/// Returns an error when the signing domain cannot be resolved or when the order digest cannot be
/// derived for the verification request.
pub fn eip1271_order_verification_request(
    order_to_sign: &cow_sdk_core::UnsignedOrder,
    chain_id: cow_sdk_core::SupportedChainId,
    verification: &crate::types::Eip1271VerificationParameters,
    options: Option<&ProtocolOptions>,
) -> Result<cow_sdk_contracts::Eip1271VerificationRequest, TradingError> {
    let domain = cow_sdk_signing::get_domain(chain_id, options)?;
    let digest =
        cow_sdk_contracts::hash_order(&domain, &cow_sdk_contracts::Order::from(order_to_sign))?;

    Ok(cow_sdk_contracts::Eip1271VerificationRequest::new(
        verification.verifier,
        digest,
        verification.signature.clone(),
    ))
}

/// Verifies an EIP-1271 order signature against a provider.
///
/// # Errors
///
/// Returns an error when the verification request cannot be derived or when the provider reports
/// missing code, malformed responses, or an invalid EIP-1271 magic value.
pub async fn verify_eip1271_order_signature<P>(
    provider: &P,
    order_to_sign: &cow_sdk_core::UnsignedOrder,
    chain_id: cow_sdk_core::SupportedChainId,
    verification: &crate::types::Eip1271VerificationParameters,
    options: Option<&ProtocolOptions>,
) -> Result<(), TradingError>
where
    P: Provider,
    P::Error: std::fmt::Display,
{
    let request =
        eip1271_order_verification_request(order_to_sign, chain_id, verification, options)?;
    let verification = cow_sdk_contracts::verify_eip1271_signature_cached(
        provider,
        &request,
        &cow_sdk_signing::NoopEip1271VerificationCache,
    );
    #[cfg(feature = "tracing")]
    let verification = {
        use tracing::Instrument as _;

        verification.instrument(tracing::debug_span!(
            "trading.verify_eip1271_caller",
            chain_id = ?chain_id,
            verifier = %request.verifier,
        ))
    };
    verification.await?;
    Ok(())
}
