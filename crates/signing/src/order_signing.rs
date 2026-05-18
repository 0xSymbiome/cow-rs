use std::fmt;
use std::str::FromStr;

use alloy_primitives::{Address as AlloyAddress, B256, Bytes as AlloyBytes, U256, keccak256};
use alloy_sol_types::SolValue;
use cow_sdk_contracts::{
    ContractsError, Order as ContractsOrder, OrderUidParams, SigningScheme, hash_order,
    normalize_order, normalized_ecdsa_signature, pack_order_uid_params,
};
use cow_sdk_core::{
    Address, AsyncDigestSigner, AsyncTypedDataSigner, BuyTokenDestination, OrderDigest, OrderKind,
    OrderUid, ProtocolOptions, SellTokenSource, Signer, SupportedChainId, TypedDataPayload,
    UnsignedOrder,
};
use serde::{Deserialize, Serialize};

use crate::eip1271::{OnchainOrder, OrderAndSignature};
use crate::{
    SigningError,
    domain::{get_domain, order_typed_data_payload},
};

/// Result of a local signing operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SigningResult {
    /// Encoded signature string.
    pub signature: String,
    /// Signing scheme used to create `signature`.
    pub signing_scheme: SigningScheme,
}

impl SigningResult {
    /// Creates the result of a local signing operation.
    #[must_use]
    pub fn new(signature: impl Into<String>, signing_scheme: SigningScheme) -> Self {
        Self {
            signature: signature.into(),
            signing_scheme,
        }
    }
}

/// Generated compact order identifier plus underlying digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GeneratedOrderId {
    /// Compact order UID.
    pub order_id: OrderUid,
    /// Underlying order digest.
    pub order_digest: OrderDigest,
}

impl GeneratedOrderId {
    /// Creates a generated compact order identifier plus underlying digest.
    #[must_use]
    pub const fn new(order_id: OrderUid, order_digest: OrderDigest) -> Self {
        Self {
            order_id,
            order_digest,
        }
    }
}

struct OrderSigningPayload {
    payload: TypedDataPayload,
    digest: String,
}

/// Signs an order using `Eip712`.
///
/// # Errors
///
/// Returns [`SigningError`] if payload construction, hashing, or signer execution fails.
pub fn sign_order<S>(
    order: &UnsignedOrder,
    chain_id: SupportedChainId,
    signer: &S,
    options: Option<&ProtocolOptions>,
) -> Result<SigningResult, SigningError>
where
    S: Signer,
    S::Error: fmt::Display,
{
    sign_order_with_scheme(order, chain_id, signer, SigningScheme::Eip712, options)
}

/// Signs an order asynchronously using `Eip712`.
///
/// # Errors
///
/// Returns [`SigningError`] if payload construction, hashing, or signer execution fails.
pub async fn sign_order_async<S>(
    order: &UnsignedOrder,
    chain_id: SupportedChainId,
    signer: &S,
    options: Option<&ProtocolOptions>,
) -> Result<SigningResult, SigningError>
where
    S: AsyncTypedDataSigner,
    S::Error: fmt::Display,
{
    let payload = order_signing_payload(order, chain_id, options)?;
    let signature = signer
        .sign_typed_data_payload(&payload.payload)
        .await
        .map_err(|error| signer_error("sign_typed_data_payload", error))?;
    Ok(SigningResult {
        signature: normalized_ecdsa_signature(&signature)?,
        signing_scheme: SigningScheme::Eip712,
    })
}

/// Signs an order using an explicit local signing scheme.
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
            endpoint = "signing.order",
        ),
    ),
)]
pub fn sign_order_with_scheme<S>(
    order: &UnsignedOrder,
    chain_id: SupportedChainId,
    signer: &S,
    scheme: SigningScheme,
    options: Option<&ProtocolOptions>,
) -> Result<SigningResult, SigningError>
where
    S: Signer,
    S::Error: fmt::Display,
{
    let payload = order_signing_payload(order, chain_id, options)?;
    sign_with_scheme(signer, scheme, &payload.payload, &payload.digest)
}

/// Signs an order asynchronously using an explicit local signing scheme.
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
            endpoint = "signing.order",
        ),
    ),
)]
pub async fn sign_order_with_scheme_async<S>(
    order: &UnsignedOrder,
    chain_id: SupportedChainId,
    signer: &S,
    scheme: SigningScheme,
    options: Option<&ProtocolOptions>,
) -> Result<SigningResult, SigningError>
where
    S: AsyncTypedDataSigner + AsyncDigestSigner<Error = <S as AsyncTypedDataSigner>::Error>,
    <S as AsyncTypedDataSigner>::Error: fmt::Display,
{
    let payload = order_signing_payload(order, chain_id, options)?;
    sign_with_scheme_async(signer, scheme, &payload.payload, &payload.digest).await
}

/// Generates the compact order UID for an order and owner.
///
/// # Errors
///
/// Returns [`SigningError`] if domain construction, hashing, or UID packing fails.
pub fn generate_order_id(
    chain_id: SupportedChainId,
    order: &UnsignedOrder,
    owner: &Address,
    options: Option<&ProtocolOptions>,
) -> Result<GeneratedOrderId, SigningError> {
    let domain = get_domain(chain_id, options)?;
    let order_digest = hash_order(&domain, &contracts_order(order))?;
    let order_id = pack_order_uid_params(&OrderUidParams::new(
        order_digest.clone(),
        owner.clone(),
        order.valid_to,
    ))?;

    Ok(GeneratedOrderId {
        order_id,
        order_digest,
    })
}

/// Encodes the `CoW` EIP-1271 verifier payload for an existing ECDSA signature.
///
/// Delegates to [`alloy_sol_types::SolValue::abi_encode`] on the
/// macro-emitted `OrderAndSignature` struct declared in the
/// `eip1271::sol_types` module. The struct mirrors the on-chain
/// `GPv2Order.Data` schema (with `bytes32` `kind`,
/// `sellTokenBalance`, and `buyTokenBalance` fields holding the
/// keccak256 of the canonical label string) plus the raw ECDSA
/// signature bytes the verifier consumes; the alloy primitive
/// composes the canonical head and dynamic-tail tuple layout
/// expected by the verifier.
///
/// # Errors
///
/// Returns [`SigningError`] if order normalization, address parsing,
/// or signature decoding fails.
pub fn eip1271_signature_payload(
    order: &UnsignedOrder,
    ecdsa_signature: &str,
) -> Result<String, SigningError> {
    let normalized = normalize_order(&contracts_order(order))?;
    let signature = normalized_ecdsa_signature(ecdsa_signature)?;
    let signature_bytes = decode_hex(&signature, "ecdsaSignature")?;

    let onchain_order = OnchainOrder {
        sellToken: parse_alloy_address(normalized.sell_token.as_str())?,
        buyToken: parse_alloy_address(normalized.buy_token.as_str())?,
        receiver: parse_alloy_address(normalized.receiver.as_str())?,
        sellAmount: biguint_to_u256("sellAmount", normalized.sell_amount.as_biguint())?,
        buyAmount: biguint_to_u256("buyAmount", normalized.buy_amount.as_biguint())?,
        validTo: normalized.valid_to,
        appData: parse_b256(normalized.app_data.as_str(), "appData")?,
        feeAmount: biguint_to_u256("feeAmount", normalized.fee_amount.as_biguint())?,
        kind: keccak256(order_kind_name(normalized.kind).as_bytes()),
        partiallyFillable: normalized.partially_fillable,
        sellTokenBalance: keccak256(sell_balance_name(normalized.sell_token_balance).as_bytes()),
        buyTokenBalance: keccak256(buy_balance_name(normalized.buy_token_balance).as_bytes()),
    };
    let payload: OrderAndSignature = (onchain_order, AlloyBytes::from(signature_bytes));

    Ok(format!("0x{}", hex::encode(payload.abi_encode_sequence())))
}

pub(crate) fn sign_with_scheme<S>(
    signer: &S,
    scheme: SigningScheme,
    payload: &TypedDataPayload,
    digest_hex: &str,
) -> Result<SigningResult, SigningError>
where
    S: Signer,
    S::Error: fmt::Display,
{
    if !scheme.is_ecdsa() {
        return Err(SigningError::UnsupportedSignerGeneratedScheme { scheme });
    }

    let signature = match scheme {
        SigningScheme::Eip712 => signer
            .sign_typed_data_payload(payload)
            .map_err(|error| signer_error("sign_typed_data_payload", error))?,
        SigningScheme::EthSign => {
            let digest = decode_hex(digest_hex, "digest")?;
            signer
                .sign_message(&digest)
                .map_err(|error| signer_error("sign_message", error))?
        }
        _ => {
            return Err(SigningError::UnsupportedSignerGeneratedScheme { scheme });
        }
    };

    Ok(SigningResult {
        signature: normalized_ecdsa_signature(&signature)?,
        signing_scheme: scheme,
    })
}

pub(crate) async fn sign_with_scheme_async<S>(
    signer: &S,
    scheme: SigningScheme,
    payload: &TypedDataPayload,
    digest_hex: &str,
) -> Result<SigningResult, SigningError>
where
    S: AsyncTypedDataSigner + AsyncDigestSigner<Error = <S as AsyncTypedDataSigner>::Error>,
    <S as AsyncTypedDataSigner>::Error: fmt::Display,
{
    if !scheme.is_ecdsa() {
        return Err(SigningError::UnsupportedSignerGeneratedScheme { scheme });
    }

    let signature = match scheme {
        SigningScheme::Eip712 => signer
            .sign_typed_data_payload(payload)
            .await
            .map_err(|error| signer_error("sign_typed_data_payload", error))?,
        SigningScheme::EthSign => {
            let digest = decode_hex(digest_hex, "digest")?;
            signer
                .sign_digest(&digest)
                .await
                .map_err(|error| signer_error("sign_message", error))?
        }
        _ => {
            return Err(SigningError::UnsupportedSignerGeneratedScheme { scheme });
        }
    };

    Ok(SigningResult {
        signature: normalized_ecdsa_signature(&signature)?,
        signing_scheme: scheme,
    })
}

fn order_signing_payload(
    order: &UnsignedOrder,
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<OrderSigningPayload, SigningError> {
    let domain = get_domain(chain_id, options)?;
    let digest = hash_order(&domain, &contracts_order(order))?;

    Ok(OrderSigningPayload {
        payload: order_typed_data_payload(chain_id, order, options)?,
        digest: digest.as_str().to_owned(),
    })
}

pub(crate) fn contracts_order(order: &UnsignedOrder) -> ContractsOrder {
    ContractsOrder::from(order)
}

pub(crate) fn signer_error<E: fmt::Display>(operation: &'static str, error: E) -> SigningError {
    SigningError::Signer {
        operation,
        message: error.to_string().into(),
    }
}

fn decode_hex(value: &str, field: &'static str) -> Result<Vec<u8>, SigningError> {
    let Some(stripped) = value.strip_prefix("0x") else {
        return Err(ContractsError::InvalidHexPrefix { field }.into());
    };
    hex::decode(stripped).map_err(|source| ContractsError::DecodeHex { field, source }.into())
}

fn parse_alloy_address(value: &str) -> Result<AlloyAddress, SigningError> {
    AlloyAddress::from_str(value).map_err(|_| {
        SigningError::from(ContractsError::InvalidDecodedLength {
            field: "address",
            expected: 20,
            actual: 0,
        })
    })
}

fn parse_b256(value: &str, field: &'static str) -> Result<B256, SigningError> {
    B256::from_str(value).map_err(|_| {
        SigningError::from(ContractsError::InvalidDecodedLength {
            field,
            expected: 32,
            actual: 0,
        })
    })
}

fn biguint_to_u256(field: &'static str, value: &num_bigint::BigUint) -> Result<U256, SigningError> {
    let bytes = value.to_bytes_be();
    if bytes.len() > 32 {
        return Err(ContractsError::NumericOverflow {
            field,
            value: value.to_str_radix(10).into(),
        }
        .into());
    }
    let mut buf = [0_u8; 32];
    buf[32 - bytes.len()..].copy_from_slice(&bytes);
    Ok(U256::from_be_bytes(buf))
}

const fn order_kind_name(kind: OrderKind) -> &'static str {
    match kind {
        OrderKind::Buy => "buy",
        OrderKind::Sell => "sell",
    }
}

/// Returns the EIP-712 label for a supported sell-token balance source.
///
/// # Panics
///
/// Panics only if a new balance-source variant reaches this signing codec
/// before the typed-data label mapping is updated.
fn sell_balance_name(balance: SellTokenSource) -> &'static str {
    match balance {
        SellTokenSource::Erc20 => "erc20",
        SellTokenSource::External => "external",
        SellTokenSource::Internal => "internal",
        // SAFETY: every currently supported signing balance source is mapped
        // above; new variants must extend this typed-data codec.
        _ => unreachable!("SellTokenSource variants are exhaustively covered"),
    }
}

/// Returns the EIP-712 label for a supported buy-token balance destination.
///
/// # Panics
///
/// Panics only if a new balance-destination variant reaches this signing codec
/// before the typed-data label mapping is updated.
fn buy_balance_name(balance: BuyTokenDestination) -> &'static str {
    match balance {
        BuyTokenDestination::Erc20 => "erc20",
        BuyTokenDestination::Internal => "internal",
        // SAFETY: every currently supported signing balance destination is
        // mapped above; new variants must extend this typed-data codec.
        _ => unreachable!("BuyTokenDestination variants are exhaustively covered"),
    }
}
