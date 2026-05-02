use std::fmt;

use cow_sdk_contracts::{
    ContractsError, Order as ContractsOrder, OrderUidParams, SigningScheme, hash_order,
    normalize_order, normalized_ecdsa_signature, pack_order_uid_params,
};
use cow_sdk_core::{
    Address, AsyncSigner, BuyTokenDestination, OrderDigest, OrderKind, OrderUid, ProtocolOptions,
    SellTokenSource, Signer, SupportedChainId, TypedDataPayload, UnsignedOrder,
};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

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
    S: AsyncSigner,
    S::Error: fmt::Display,
{
    sign_order_with_scheme_async(order, chain_id, signer, SigningScheme::Eip712, options).await
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
    S: AsyncSigner,
    S::Error: fmt::Display,
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
/// # Errors
///
/// Returns [`SigningError`] if order normalization or ABI-style encoding fails.
pub fn eip1271_signature_payload(
    order: &UnsignedOrder,
    ecdsa_signature: &str,
) -> Result<String, SigningError> {
    let normalized_order = normalize_order(&contracts_order(order))?;
    let signature = normalized_ecdsa_signature(ecdsa_signature)?;
    let signature_bytes = parse_hex(&signature, "ecdsaSignature")?;

    let mut encoded = Vec::with_capacity(32 * 13 + 32 + signature_bytes.len() + 32);
    encoded.extend_from_slice(&encode_address(normalized_order.sell_token.as_str())?);
    encoded.extend_from_slice(&encode_address(normalized_order.buy_token.as_str())?);
    encoded.extend_from_slice(&encode_address(normalized_order.receiver.as_str())?);
    encoded.extend_from_slice(&encode_u256_str(
        "sellAmount",
        &normalized_order.sell_amount.to_string(),
    )?);
    encoded.extend_from_slice(&encode_u256_str(
        "buyAmount",
        &normalized_order.buy_amount.to_string(),
    )?);
    encoded.extend_from_slice(&encode_u32(normalized_order.valid_to));
    encoded.extend_from_slice(&parse_hex32(normalized_order.app_data.as_str(), "appData")?);
    encoded.extend_from_slice(&encode_u256_str(
        "feeAmount",
        &normalized_order.fee_amount.to_string(),
    )?);
    encoded.extend_from_slice(&keccak256(
        order_kind_name(normalized_order.kind).as_bytes(),
    ));
    encoded.extend_from_slice(&encode_bool(normalized_order.partially_fillable));
    encoded.extend_from_slice(&keccak256(
        sell_balance_name(normalized_order.sell_token_balance).as_bytes(),
    ));
    encoded.extend_from_slice(&keccak256(
        buy_balance_name(normalized_order.buy_token_balance).as_bytes(),
    ));
    encoded.extend_from_slice(&encode_usize_u256(32 * 13));
    encoded.extend_from_slice(&encode_usize_u256(signature_bytes.len()));
    encoded.extend_from_slice(&signature_bytes);
    encoded.extend(std::iter::repeat_n(
        0u8,
        padded_len(signature_bytes.len()) - signature_bytes.len(),
    ));

    Ok(format!("0x{}", hex::encode(encoded)))
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
            let digest = parse_hex(digest_hex, "digest")?;
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
    S: AsyncSigner,
    S::Error: fmt::Display,
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
            let digest = parse_hex(digest_hex, "digest")?;
            signer
                .sign_message(&digest)
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

fn signer_error<E: fmt::Display>(operation: &'static str, error: E) -> SigningError {
    SigningError::Signer {
        operation,
        message: error.to_string(),
    }
}

fn parse_hex(value: &str, field: &'static str) -> Result<Vec<u8>, SigningError> {
    let Some(stripped) = value.strip_prefix("0x") else {
        return Err(ContractsError::InvalidHexPrefix { field }.into());
    };

    hex::decode(stripped).map_err(|source| ContractsError::DecodeHex { field, source }.into())
}

fn parse_hex32(value: &str, field: &'static str) -> Result<[u8; 32], SigningError> {
    let bytes = parse_hex(value, field)?;
    if bytes.len() != 32 {
        return Err(ContractsError::InvalidDecodedLength {
            field,
            expected: 32,
            actual: bytes.len(),
        }
        .into());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn encode_address(value: &str) -> Result<[u8; 32], SigningError> {
    let bytes = parse_hex(value, "address")?;
    if bytes.len() != 20 {
        return Err(ContractsError::InvalidDecodedLength {
            field: "address",
            expected: 20,
            actual: bytes.len(),
        }
        .into());
    }
    let mut out = [0u8; 32];
    out[12..].copy_from_slice(&bytes);
    Ok(out)
}

fn encode_u32(value: u32) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[28..].copy_from_slice(&value.to_be_bytes());
    out
}

fn encode_bool(value: bool) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[31] = u8::from(value);
    out
}

fn encode_u256_str(field: &'static str, value: &str) -> Result<[u8; 32], SigningError> {
    let parsed = value
        .strip_prefix("0x")
        .map_or_else(
            || BigUint::parse_bytes(value.as_bytes(), 10),
            |stripped| BigUint::parse_bytes(stripped.as_bytes(), 16),
        )
        .ok_or_else(|| ContractsError::InvalidNumeric {
            field,
            value: value.to_owned(),
        })?;

    let bytes = parsed.to_bytes_be();
    if bytes.len() > 32 {
        return Err(ContractsError::NumericOverflow {
            field,
            value: value.to_owned(),
        }
        .into());
    }

    let mut out = [0u8; 32];
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    Ok(out)
}

fn encode_usize_u256(value: usize) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[24..].copy_from_slice(&(value as u64).to_be_bytes());
    out
}

const fn padded_len(len: usize) -> usize {
    if len == 0 {
        0
    } else {
        ((len - 1) / 32 + 1) * 32
    }
}

fn keccak256(bytes: impl AsRef<[u8]>) -> [u8; 32] {
    let digest = Keccak256::digest(bytes.as_ref());
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
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
