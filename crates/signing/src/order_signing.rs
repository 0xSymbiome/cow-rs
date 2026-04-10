use std::fmt;

use cow_sdk_contracts::{
    ContractsError, Order as ContractsOrder, OrderUidParams, SigningScheme, hash_order,
    normalize_order, normalized_ecdsa_signature, pack_order_uid_params,
};
use cow_sdk_core::{
    Address, AsyncSigner, OrderBalance, OrderKind, OrderUid, ProtocolOptions, Signer,
    SupportedChainId, TypedDataDomain, TypedDataField, UnsignedOrder,
};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

use crate::{
    SigningError,
    domain::{get_domain, order_fields},
};

pub type TypedOrder = UnsignedOrder;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SigningResult {
    pub signature: String,
    pub signing_scheme: SigningScheme,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedOrderId {
    pub order_id: OrderUid,
    pub order_digest: String,
}

struct OrderSigningPayload {
    domain: TypedDataDomain,
    fields: Vec<TypedDataField>,
    value_json: String,
    digest: String,
}

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
    sign_with_scheme(
        signer,
        scheme,
        &payload.domain,
        &payload.fields,
        &payload.value_json,
        &payload.digest,
    )
}

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
    sign_with_scheme_async(
        signer,
        scheme,
        &payload.domain,
        &payload.fields,
        &payload.value_json,
        &payload.digest,
    )
    .await
}

pub fn generate_order_id(
    chain_id: SupportedChainId,
    order: &UnsignedOrder,
    owner: &Address,
    options: Option<&ProtocolOptions>,
) -> Result<GeneratedOrderId, SigningError> {
    let domain = get_domain(chain_id, options)?;
    let order_digest = hash_order(&domain, &contracts_order(order))?;
    let order_id = pack_order_uid_params(&OrderUidParams {
        order_digest: order_digest.clone(),
        owner: owner.clone(),
        valid_to: order.valid_to,
    })?;

    Ok(GeneratedOrderId {
        order_id,
        order_digest,
    })
}

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
        &normalized_order.sell_amount,
    )?);
    encoded.extend_from_slice(&encode_u256_str("buyAmount", &normalized_order.buy_amount)?);
    encoded.extend_from_slice(&encode_u32(normalized_order.valid_to));
    encoded.extend_from_slice(&parse_hex32(normalized_order.app_data.as_str(), "appData")?);
    encoded.extend_from_slice(&encode_u256_str("feeAmount", &normalized_order.fee_amount)?);
    encoded.extend_from_slice(&keccak256(
        order_kind_name(normalized_order.kind).as_bytes(),
    ));
    encoded.extend_from_slice(&encode_bool(normalized_order.partially_fillable));
    encoded.extend_from_slice(&keccak256(
        balance_name(normalized_order.sell_token_balance).as_bytes(),
    ));
    encoded.extend_from_slice(&keccak256(
        balance_name(normalized_order.buy_token_balance).as_bytes(),
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
    domain: &TypedDataDomain,
    fields: &[TypedDataField],
    value_json: &str,
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
            .sign_typed_data(domain, fields, value_json)
            .map_err(|error| signer_error("sign_typed_data", error))?,
        SigningScheme::EthSign => {
            let digest = parse_hex(digest_hex, "digest")?;
            signer
                .sign_message(&digest)
                .map_err(|error| signer_error("sign_message", error))?
        }
        SigningScheme::Eip1271 | SigningScheme::PreSign => {
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
    domain: &TypedDataDomain,
    fields: &[TypedDataField],
    value_json: &str,
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
            .sign_typed_data(domain, fields, value_json)
            .await
            .map_err(|error| signer_error("sign_typed_data", error))?,
        SigningScheme::EthSign => {
            let digest = parse_hex(digest_hex, "digest")?;
            signer
                .sign_message(&digest)
                .await
                .map_err(|error| signer_error("sign_message", error))?
        }
        SigningScheme::Eip1271 | SigningScheme::PreSign => {
            return Err(SigningError::UnsupportedSignerGeneratedScheme { scheme });
        }
    };

    Ok(SigningResult {
        signature: normalized_ecdsa_signature(&signature)?,
        signing_scheme: scheme,
    })
}

pub(crate) fn serialize<T: Serialize>(value: &T) -> Result<String, SigningError> {
    serde_json::to_string(value).map_err(|error| SigningError::Serialization(error.to_string()))
}

fn order_signing_payload(
    order: &UnsignedOrder,
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<OrderSigningPayload, SigningError> {
    let domain = get_domain(chain_id, options)?;
    let value_json = serialize(order)?;
    let digest = hash_order(&domain, &contracts_order(order))?;

    Ok(OrderSigningPayload {
        domain,
        fields: order_fields(),
        value_json,
        digest,
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
        return Err(ContractsError::Decode(format!(
            "{field} must be 0x-prefixed hexadecimal data"
        ))
        .into());
    };

    hex::decode(stripped)
        .map_err(|_| ContractsError::Decode(format!("{field} contains non-hex characters")).into())
}

fn parse_hex32(value: &str, field: &'static str) -> Result<[u8; 32], SigningError> {
    let bytes = parse_hex(value, field)?;
    if bytes.len() != 32 {
        return Err(ContractsError::Decode(format!(
            "{field} must be 32 bytes, got {}",
            bytes.len()
        ))
        .into());
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn encode_address(value: &str) -> Result<[u8; 32], SigningError> {
    let bytes = parse_hex(value, "address")?;
    if bytes.len() != 20 {
        return Err(ContractsError::Decode(format!(
            "address must be 20 bytes, got {}",
            bytes.len()
        ))
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
    let parsed = if let Some(stripped) = value.strip_prefix("0x") {
        BigUint::parse_bytes(stripped.as_bytes(), 16)
    } else {
        BigUint::parse_bytes(value.as_bytes(), 10)
    }
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

fn padded_len(len: usize) -> usize {
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

fn order_kind_name(kind: OrderKind) -> &'static str {
    match kind {
        OrderKind::Buy => "buy",
        OrderKind::Sell => "sell",
    }
}

fn balance_name(balance: OrderBalance) -> &'static str {
    match balance {
        OrderBalance::Erc20 => "erc20",
        OrderBalance::External => "external",
        OrderBalance::Internal => "internal",
    }
}
