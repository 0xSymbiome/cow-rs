use std::fmt;

use alloy_primitives::{Bytes as AlloyBytes, keccak256};
use alloy_sol_types::SolValue;
use cow_sdk_contracts::{
    ContractsError, Order as ContractsOrder, OrderUidParams, SigningScheme, buy_balance_name,
    hash_order, normalize_order, normalized_ecdsa_signature, order_kind_name,
    pack_order_uid_params, sell_balance_name,
};
use cow_sdk_core::{
    Address, AsyncDigestSigner, AsyncTypedDataSigner, OrderDigest, OrderUid, ProtocolOptions,
    Signer, SignerError, SupportedChainId, TypedDataPayload, UnsignedOrder,
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
    S::Error: fmt::Display + SignerError,
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
    S::Error: fmt::Display + SignerError,
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
    S::Error: fmt::Display + SignerError,
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
    <S as AsyncTypedDataSigner>::Error: fmt::Display + SignerError,
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
    let order_id =
        pack_order_uid_params(&OrderUidParams::new(order_digest, *owner, order.valid_to))?;

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

    // The cow `Amount` newtype is `#[repr(transparent)]` over
    // `alloy_primitives::U256` and `AppDataHash` over
    // `alloy_primitives::B256` per ADR 0052, so the conversions to the
    // sol-typed surface are a single deref of the inner alloy primitive
    // with no intermediate bigint allocation and no overflow guard
    // required.
    let onchain_order = OnchainOrder {
        sellToken: *normalized.sell_token.as_alloy(),
        buyToken: *normalized.buy_token.as_alloy(),
        receiver: *normalized.receiver.as_alloy(),
        sellAmount: *normalized.sell_amount.as_u256(),
        buyAmount: *normalized.buy_amount.as_u256(),
        validTo: normalized.valid_to,
        appData: *normalized.app_data.as_alloy(),
        feeAmount: *normalized.fee_amount.as_u256(),
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
    S::Error: fmt::Display + SignerError,
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
    <S as AsyncTypedDataSigner>::Error: fmt::Display + SignerError,
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
        digest: digest.to_hex_string(),
    })
}

pub(crate) fn contracts_order(order: &UnsignedOrder) -> ContractsOrder {
    ContractsOrder::from(order)
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "callers move the upstream signer error into this helper at the \
              `.map_err` boundary; consuming the value keeps the failure path \
              free of additional borrows."
)]
pub(crate) fn signer_error<E: fmt::Display + SignerError>(
    operation: &'static str,
    error: E,
) -> SigningError {
    if let Some(code) = error.user_rejection_code() {
        return SigningError::SignerRejection {
            label: signer_operation_label(operation),
            code,
        };
    }
    SigningError::Signer {
        operation,
        message: error.to_string().into(),
    }
}

/// Maps the static signing-helper call-site identifier to the
/// human-readable operation label rendered in
/// [`SigningError::SignerRejection`]. The labels are intentionally
/// concise and product-facing so downstream consoles can render the
/// rejection without inspecting backend-specific strings.
fn signer_operation_label(operation: &str) -> &'static str {
    match operation {
        "sign_typed_data_payload" | "sign_typed_data" => "typed-data signature",
        "sign_message" | "sign_digest" => "message signature",
        _ => "signing request",
    }
}

fn decode_hex(value: &str, field: &'static str) -> Result<Vec<u8>, SigningError> {
    let Some(stripped) = value.strip_prefix("0x") else {
        return Err(ContractsError::InvalidHexPrefix { field }.into());
    };
    hex::decode(stripped).map_err(|source| ContractsError::DecodeHex { field, source }.into())
}

#[cfg(test)]
mod signer_error_tests {
    use std::{collections::BTreeMap, sync::Mutex};

    use super::*;

    /// Minimal typed signer error used to exercise the `signer_error`
    /// helper against the [`SignerError`] trait without pulling in
    /// any downstream signer crate. The wrapper carries an optional
    /// rejection code so each test pins the exact classification the
    /// trait should expose for the helper to consume.
    #[derive(Debug, Clone)]
    struct FakeSignerError {
        message: &'static str,
        rejection_code: Option<i32>,
    }

    impl fmt::Display for FakeSignerError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.message)
        }
    }

    impl SignerError for FakeSignerError {
        fn user_rejection_code(&self) -> Option<i32> {
            self.rejection_code
        }
    }

    #[derive(Default)]
    struct RecordingAsyncSigner {
        typed_data_messages: Mutex<Vec<String>>,
        digest_messages: Mutex<Vec<Vec<u8>>>,
    }

    impl AsyncTypedDataSigner for RecordingAsyncSigner {
        type Error = FakeSignerError;

        async fn sign_typed_data(
            &self,
            _domain: &cow_sdk_core::TypedDataDomain,
            _fields: &[cow_sdk_core::TypedDataField],
            value_json: &str,
        ) -> Result<String, Self::Error> {
            self.typed_data_messages
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(value_json.to_owned());
            Ok(test_signature("aa"))
        }
    }

    impl AsyncDigestSigner for RecordingAsyncSigner {
        type Error = FakeSignerError;

        async fn sign_digest(&self, digest: &[u8]) -> Result<String, Self::Error> {
            self.digest_messages
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(digest.to_vec());
            Ok(test_signature("bb"))
        }
    }

    fn test_signature(byte: &str) -> String {
        format!("0x{}1b", byte.repeat(64))
    }

    fn test_payload() -> TypedDataPayload {
        let domain = cow_sdk_core::TypedDataDomain::new(
            "Gnosis Protocol".to_owned(),
            "v2".to_owned(),
            1,
            Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap(),
        );
        let fields = vec![cow_sdk_core::TypedDataField::new(
            "sellToken".to_owned(),
            "address".to_owned(),
        )];
        TypedDataPayload::new(
            domain,
            "Order".to_owned(),
            BTreeMap::from([("Order".to_owned(), fields)]),
            "{\"sellToken\":\"0x1111111111111111111111111111111111111111\"}".to_owned(),
        )
    }

    #[tokio::test]
    async fn async_sign_with_scheme_routes_eip712_to_typed_data_signer() {
        let signer = RecordingAsyncSigner::default();
        let payload = test_payload();

        let result = sign_with_scheme_async(&signer, SigningScheme::Eip712, &payload, "0x")
            .await
            .expect("EIP-712 async signing must use the typed-data signer path");

        assert_eq!(result.signing_scheme, SigningScheme::Eip712);
        assert_eq!(result.signature, test_signature("aa"));
        assert_eq!(
            signer
                .typed_data_messages
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_slice(),
            ["{\"sellToken\":\"0x1111111111111111111111111111111111111111\"}"],
        );
        assert!(
            signer
                .digest_messages
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty(),
            "EIP-712 signing must not route through digest signing",
        );
    }

    #[test]
    fn signer_error_routes_typed_rejection_to_structured_variant() {
        let upstream = FakeSignerError {
            message: "wallet rejected; opaque",
            rejection_code: Some(4001),
        };
        let error = signer_error("sign_typed_data_payload", upstream);
        match error {
            SigningError::SignerRejection { label, code } => {
                assert_eq!(label, "typed-data signature");
                assert_eq!(code, 4001);
            }
            other => panic!("expected SignerRejection, got {other:?}"),
        }
    }

    #[test]
    fn signer_error_propagates_non_4001_rejection_codes_verbatim() {
        let upstream = FakeSignerError {
            message: "wallet account disconnected",
            rejection_code: Some(4900),
        };
        let error = signer_error("sign_typed_data_payload", upstream);
        match error {
            SigningError::SignerRejection { label, code } => {
                assert_eq!(label, "typed-data signature");
                assert_eq!(code, 4900);
            }
            other => panic!("expected SignerRejection, got {other:?}"),
        }
    }

    #[test]
    fn signer_error_falls_back_to_redacted_signer_for_unclassified_errors() {
        let upstream = FakeSignerError {
            message: "signer hardware module busy",
            rejection_code: None,
        };
        let error = signer_error("sign_typed_data_payload", upstream);
        match error {
            SigningError::Signer { operation, message } => {
                assert_eq!(operation, "sign_typed_data_payload");
                assert_eq!(message.as_inner(), "signer hardware module busy");
            }
            other => panic!("expected Signer, got {other:?}"),
        }
    }

    #[test]
    fn signer_rejection_display_renders_user_facing_label_and_code() {
        let rendered = SigningError::SignerRejection {
            label: "typed-data signature",
            code: 4001,
        }
        .to_string();
        assert!(rendered.contains("User rejected typed-data signature"));
        assert!(rendered.contains("(4001)"));
    }

    #[test]
    fn signer_operation_label_maps_known_operations() {
        assert_eq!(
            signer_operation_label("sign_typed_data_payload"),
            "typed-data signature"
        );
        assert_eq!(
            signer_operation_label("sign_typed_data"),
            "typed-data signature"
        );
        assert_eq!(signer_operation_label("sign_message"), "message signature");
        assert_eq!(signer_operation_label("sign_digest"), "message signature");
        assert_eq!(signer_operation_label("sign_unknown_op"), "signing request");
    }
}
