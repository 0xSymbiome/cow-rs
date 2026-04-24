use serde::{Deserialize, Serialize};

use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, Hash32, OrderDigest, OrderKind, OrderUid,
    SellTokenSource, TypedDataDomain,
};

use crate::{
    ContractsError,
    primitives::{
        ORDER_UID_LENGTH_BYTES, buy_balance_name, encode_address, encode_bool, encode_string_hash,
        encode_u32, encode_u256_biguint, keccak256, order_kind_name, parse_bytes32_hash,
        parse_hex_exact, parse_hex32, sell_balance_name, typed_data_digest, zero_address,
    },
};

/// Sentinel address used by the protocol to represent native ETH buys.
pub const BUY_ETH_ADDRESS: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
/// EIP-712 order type hash used for struct hashing.
pub const ORDER_TYPE_HASH: &str =
    "0xd5a25ba2e97094ad7d83dc28a6572da797d6b3e7fc6663bd93efb789fc17e489";
/// Encoded order UID length in bytes.
pub const ORDER_UID_LENGTH: usize = ORDER_UID_LENGTH_BYTES;

/// EIP-712 field descriptor used for `CoW` order-type metadata.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderTypeField {
    /// Field name.
    pub name: &'static str,
    /// Solidity field type.
    #[serde(rename = "type")]
    pub kind: &'static str,
}

impl OrderTypeField {
    /// Creates an order-type field descriptor.
    #[must_use]
    pub const fn new(name: &'static str, kind: &'static str) -> Self {
        Self { name, kind }
    }
}

/// Canonical order type fields in struct-hash order.
pub const ORDER_TYPE_FIELDS: [OrderTypeField; 12] = [
    OrderTypeField::new("sellToken", "address"),
    OrderTypeField::new("buyToken", "address"),
    OrderTypeField::new("receiver", "address"),
    OrderTypeField::new("sellAmount", "uint256"),
    OrderTypeField::new("buyAmount", "uint256"),
    OrderTypeField::new("validTo", "uint32"),
    OrderTypeField::new("appData", "bytes32"),
    OrderTypeField::new("feeAmount", "uint256"),
    OrderTypeField::new("kind", "string"),
    OrderTypeField::new("partiallyFillable", "bool"),
    OrderTypeField::new("sellTokenBalance", "string"),
    OrderTypeField::new("buyTokenBalance", "string"),
];

/// Canonical EIP-712 field descriptor for order-cancellation payloads.
pub const CANCELLATIONS_TYPE_FIELDS: [OrderTypeField; 1] =
    [OrderTypeField::new("orderUids", "bytes[]")];

/// Contract ABI and EIP-712 order payload.
///
/// This type intentionally differs from `cow_sdk_core::UnsignedOrder`: receiver
/// and token-balance fields are optional here because the contract hashing
/// boundary applies `CoW` Protocol defaults during normalization.
///
/// Convert user-domain orders into this type through the canonical
/// [`cow_sdk_core::UnsignedOrder`] boundary.
///
/// ```
/// use cow_sdk_contracts::Order;
/// use cow_sdk_core::{
///     Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, SellTokenSource,
///     UnsignedOrder,
/// };
///
/// let unsigned = UnsignedOrder::new(
///     Address::new("0x1111111111111111111111111111111111111111").unwrap(),
///     Address::new("0x2222222222222222222222222222222222222222").unwrap(),
///     Address::new("0x3333333333333333333333333333333333333333").unwrap(),
///     Amount::new("100").unwrap(),
///     Amount::new("200").unwrap(),
///     1_700_000_000,
///     AppDataHash::new(
///         "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
///     )
///     .unwrap(),
///     Amount::new("5").unwrap(),
///     OrderKind::Sell,
///     false,
///     SellTokenSource::External,
///     BuyTokenDestination::Internal,
/// );
///
/// let order = Order::from(&unsigned);
/// assert_eq!(order.valid_to, unsigned.valid_to);
/// assert_eq!(order.fee_amount, unsigned.fee_amount);
/// ```
#[doc = concat!(
    "```compile_fail\n",
    "use cow_sdk_contracts::{hash_order_for_", "contract, uid_for_", "contract};\n",
    "use cow_sdk_core::{Order", "Model, Quote", "Model};\n",
    "\n",
    "fn main() {}\n",
    "```\n",
)]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Sell token address.
    pub sell_token: Address,
    /// Buy token address.
    pub buy_token: Address,
    /// Optional receiver. Missing values normalize to `address(0)`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Sell amount.
    pub sell_amount: Amount,
    /// Buy amount.
    pub buy_amount: Amount,
    /// Expiration timestamp.
    pub valid_to: u32,
    /// App-data hash.
    pub app_data: AppDataHash,
    /// Fee amount.
    pub fee_amount: Amount,
    /// Order side.
    pub kind: OrderKind,
    /// Whether the order is partially fillable.
    pub partially_fillable: bool,
    /// Optional sell-token balance source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<SellTokenSource>,
    /// Optional buy-token balance destination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<BuyTokenDestination>,
}

/// Canonical contract order used for struct hashing.
///
/// `normalize_order` creates this type after applying ABI-level defaults and
/// rejecting invalid receiver state. It is separate from [`Order`] so hashing code
/// cannot accidentally skip normalization.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedOrder {
    /// Sell token address.
    pub sell_token: Address,
    /// Buy token address.
    pub buy_token: Address,
    /// Normalized receiver address.
    pub receiver: Address,
    /// Sell amount.
    pub sell_amount: Amount,
    /// Buy amount.
    pub buy_amount: Amount,
    /// Expiration timestamp.
    pub valid_to: u32,
    /// App-data hash.
    pub app_data: AppDataHash,
    /// Fee amount.
    pub fee_amount: Amount,
    /// Order side.
    pub kind: OrderKind,
    /// Whether the order is partially fillable.
    pub partially_fillable: bool,
    /// Normalized sell-token balance source.
    pub sell_token_balance: SellTokenSource,
    /// Normalized buy-token balance destination.
    pub buy_token_balance: BuyTokenDestination,
}

/// Structured order UID components.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderUidParams {
    /// Order digest.
    pub order_digest: OrderDigest,
    /// Order owner address.
    pub owner: Address,
    /// Order expiration timestamp.
    pub valid_to: u32,
}

/// EIP-712 message body for order cancellations.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCancellations {
    /// Order UIDs being cancelled.
    pub order_uids: Vec<OrderUid>,
}

impl Order {
    /// Creates a contract order payload.
    #[must_use]
    // Mirrors the full current public field set so callers can migrate off
    // struct literals without losing explicit control over any wire field.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        sell_token: Address,
        buy_token: Address,
        receiver: Option<Address>,
        sell_amount: Amount,
        buy_amount: Amount,
        valid_to: u32,
        app_data: AppDataHash,
        fee_amount: Amount,
        kind: OrderKind,
        partially_fillable: bool,
        sell_token_balance: Option<SellTokenSource>,
        buy_token_balance: Option<BuyTokenDestination>,
    ) -> Self {
        Self {
            sell_token,
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            valid_to,
            app_data,
            fee_amount,
            kind,
            partially_fillable,
            sell_token_balance,
            buy_token_balance,
        }
    }

    /// Returns the normalized contract order used for hashing and encoding.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::ZeroReceiver`] when the receiver is explicitly
    /// set to the zero address.
    pub fn normalize(&self) -> Result<NormalizedOrder, ContractsError> {
        normalize_order(self)
    }
}

impl NormalizedOrder {
    /// Creates a normalized contract order payload.
    #[must_use]
    // Mirrors the full current public field set so callers can migrate off
    // struct literals without losing explicit control over any wire field.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        sell_token: Address,
        buy_token: Address,
        receiver: Address,
        sell_amount: Amount,
        buy_amount: Amount,
        valid_to: u32,
        app_data: AppDataHash,
        fee_amount: Amount,
        kind: OrderKind,
        partially_fillable: bool,
        sell_token_balance: SellTokenSource,
        buy_token_balance: BuyTokenDestination,
    ) -> Self {
        Self {
            sell_token,
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            valid_to,
            app_data,
            fee_amount,
            kind,
            partially_fillable,
            sell_token_balance,
            buy_token_balance,
        }
    }
}

impl OrderUidParams {
    /// Creates structured order UID components.
    #[must_use]
    pub const fn new(order_digest: OrderDigest, owner: Address, valid_to: u32) -> Self {
        Self {
            order_digest,
            owner,
            valid_to,
        }
    }
}

impl OrderCancellations {
    /// Creates an order-cancellation payload.
    #[must_use]
    pub const fn new(order_uids: Vec<OrderUid>) -> Self {
        Self { order_uids }
    }
}

impl From<&cow_sdk_core::UnsignedOrder> for Order {
    fn from(order: &cow_sdk_core::UnsignedOrder) -> Self {
        Self::new(
            order.sell_token.clone(),
            order.buy_token.clone(),
            Some(order.receiver.clone()),
            order.sell_amount.clone(),
            order.buy_amount.clone(),
            order.valid_to,
            order.app_data.clone(),
            order.fee_amount.clone(),
            order.kind,
            order.partially_fillable,
            Some(order.sell_token_balance),
            Some(order.buy_token_balance),
        )
    }
}

/// Normalizes an order into its canonical contract hashing form.
///
/// # Errors
///
/// Returns [`ContractsError::ZeroReceiver`] when the receiver is explicitly set
/// to the zero address.
pub fn normalize_order(order: &Order) -> Result<NormalizedOrder, ContractsError> {
    if matches!(
        order
            .receiver
            .as_ref()
            .map(Address::normalized_key)
            .as_deref(),
        Some(ZERO_ADDRESS_LOWER)
    ) {
        return Err(ContractsError::ZeroReceiver);
    }

    Ok(NormalizedOrder::new(
        order.sell_token.clone(),
        order.buy_token.clone(),
        order.receiver.clone().unwrap_or_else(zero_address),
        order.sell_amount.clone(),
        order.buy_amount.clone(),
        order.valid_to,
        order.app_data.clone(),
        order.fee_amount.clone(),
        order.kind,
        order.partially_fillable,
        order.sell_token_balance.unwrap_or_default(),
        order.buy_token_balance.unwrap_or_default(),
    ))
}

/// Computes the EIP-712 digest for an order.
///
/// # Errors
///
/// Returns [`ContractsError`] if normalization or struct hashing fails.
pub fn hash_order(domain: &TypedDataDomain, order: &Order) -> Result<OrderDigest, ContractsError> {
    let digest = typed_data_digest(domain, order_struct_hash(&normalize_order(order)?)?)?;
    OrderDigest::new(format!("0x{}", hex::encode(digest))).map_err(Into::into)
}

/// Computes the EIP-712 digest for a single order cancellation.
///
/// # Errors
///
/// Returns [`ContractsError`] if UID decoding or typed-data hashing fails.
pub fn hash_order_cancellation(
    domain: &TypedDataDomain,
    order_uid: &OrderUid,
) -> Result<Hash32, ContractsError> {
    hash_order_cancellations(domain, &OrderCancellations::new(vec![order_uid.clone()]))
}

/// Computes the EIP-712 digest for a batch order cancellation payload.
///
/// # Errors
///
/// Returns [`ContractsError`] if UID decoding or typed-data hashing fails.
pub fn hash_order_cancellations(
    domain: &TypedDataDomain,
    cancellations: &OrderCancellations,
) -> Result<Hash32, ContractsError> {
    let type_hash = keccak256("OrderCancellations(bytes[] orderUids)".as_bytes());
    let mut concatenated = Vec::with_capacity(cancellations.order_uids.len() * 32);
    for uid in &cancellations.order_uids {
        let bytes = parse_hex_exact(uid.as_str(), "orderUid", ORDER_UID_LENGTH)?;
        concatenated.extend_from_slice(&keccak256(bytes));
    }
    let array_hash = keccak256(concatenated);

    let mut encoded = Vec::with_capacity(64);
    encoded.extend_from_slice(&type_hash);
    encoded.extend_from_slice(&array_hash);
    let digest = typed_data_digest(domain, keccak256(encoded))?;
    Hash32::new(format!("0x{}", hex::encode(digest))).map_err(Into::into)
}

/// Computes the encoded order UID for an order and owner.
///
/// # Errors
///
/// Returns [`ContractsError`] if order hashing or UID packing fails.
#[inline]
pub fn compute_order_uid(
    domain: &TypedDataDomain,
    order: &Order,
    owner: &Address,
) -> Result<OrderUid, ContractsError> {
    pack_order_uid_params(&OrderUidParams::new(
        hash_order(domain, order)?,
        owner.clone(),
        order.valid_to,
    ))
}

/// Packs structured order UID components into the compact UID string.
///
/// # Errors
///
/// Returns [`ContractsError`] if the digest or owner cannot be decoded into the
/// fixed byte lengths required by the UID format.
#[inline]
pub fn pack_order_uid_params(params: &OrderUidParams) -> Result<OrderUid, ContractsError> {
    let digest = parse_hex32(params.order_digest.as_str(), "orderDigest")?;
    let owner = parse_hex_exact(params.owner.as_str(), "owner", 20)?;
    let mut out = [0u8; ORDER_UID_LENGTH];
    out[..32].copy_from_slice(&digest);
    out[32..52].copy_from_slice(&owner);
    out[52..56].copy_from_slice(&params.valid_to.to_be_bytes());
    OrderUid::new(format!("0x{}", hex::encode(out))).map_err(Into::into)
}

/// Extracts structured order UID components from a compact UID string.
///
/// # Errors
///
/// Returns [`ContractsError`] if the UID cannot be decoded into the expected format.
#[inline]
pub fn extract_order_uid_params(order_uid: &OrderUid) -> Result<OrderUidParams, ContractsError> {
    let bytes = parse_hex_exact(order_uid.as_str(), "orderUid", ORDER_UID_LENGTH)?;
    if bytes.len() != ORDER_UID_LENGTH {
        return Err(ContractsError::InvalidOrderUidLength {
            actual: bytes.len(),
        });
    }

    let order_digest = OrderDigest::new(format!("0x{}", hex::encode(&bytes[..32])))?;
    let owner = Address::new(format!("0x{}", hex::encode(&bytes[32..52])))?;
    let valid_to_bytes: [u8; 4] =
        bytes[52..56]
            .try_into()
            .map_err(|_| ContractsError::InvalidOrderUidLength {
                actual: bytes.len(),
            })?;
    let valid_to = u32::from_be_bytes(valid_to_bytes);

    Ok(OrderUidParams::new(order_digest, owner, valid_to))
}

fn order_struct_hash(order: &NormalizedOrder) -> Result<[u8; 32], ContractsError> {
    let mut encoded = Vec::with_capacity(32 * 13);
    encoded.extend_from_slice(&parse_hex32(ORDER_TYPE_HASH, "orderTypeHash")?);
    encoded.extend_from_slice(&encode_address(&order.sell_token)?);
    encoded.extend_from_slice(&encode_address(&order.buy_token)?);
    encoded.extend_from_slice(&encode_address(&order.receiver)?);
    encoded.extend_from_slice(&encode_u256_biguint(order.sell_amount.as_biguint())?);
    encoded.extend_from_slice(&encode_u256_biguint(order.buy_amount.as_biguint())?);
    encoded.extend_from_slice(&encode_u32(order.valid_to));
    encoded.extend_from_slice(&parse_bytes32_hash(&order.app_data)?);
    encoded.extend_from_slice(&encode_u256_biguint(order.fee_amount.as_biguint())?);
    encoded.extend_from_slice(&encode_string_hash(order_kind_name(order.kind)));
    encoded.extend_from_slice(&encode_bool(order.partially_fillable));
    encoded.extend_from_slice(&encode_string_hash(sell_balance_name(
        order.sell_token_balance,
    )));
    encoded.extend_from_slice(&encode_string_hash(buy_balance_name(
        order.buy_token_balance,
    )));
    Ok(keccak256(encoded))
}

const ZERO_ADDRESS_LOWER: &str = "0x0000000000000000000000000000000000000000";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deployments::{ContractId, Registry};
    use cow_sdk_core::{CowEnv, SupportedChainId};
    use num_bigint::BigUint;
    use sha3::{Digest, Keccak256};

    fn sample_domain() -> TypedDataDomain {
        TypedDataDomain::new(
            "Gnosis Protocol".to_owned(),
            "v2".to_owned(),
            1,
            Registry::default()
                .address(
                    ContractId::Settlement,
                    SupportedChainId::Mainnet,
                    CowEnv::Prod,
                )
                .expect("canonical settlement address is registered for every supported chain"),
        )
    }

    fn sample_order() -> Order {
        Order::new(
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            Address::new("0x2222222222222222222222222222222222222222").unwrap(),
            None,
            Amount::new("1000").unwrap(),
            Amount::new("900").unwrap(),
            1_700_000_000,
            AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap(),
            Amount::new("10").unwrap(),
            OrderKind::Sell,
            true,
            Some(SellTokenSource::External),
            Some(BuyTokenDestination::Internal),
        )
    }

    fn encode_address_word(address: &Address) -> [u8; 32] {
        let mut out = [0u8; 32];
        let decoded = hex::decode(address.as_str().trim_start_matches("0x")).unwrap();
        out[12..].copy_from_slice(&decoded);
        out
    }

    fn encode_u256_word(value: &str) -> [u8; 32] {
        let parsed = value
            .strip_prefix("0x")
            .map_or_else(
                || BigUint::parse_bytes(value.as_bytes(), 10),
                |stripped| BigUint::parse_bytes(stripped.as_bytes(), 16),
            )
            .unwrap();
        let bytes = parsed.to_bytes_be();
        let mut out = [0u8; 32];
        out[32 - bytes.len()..].copy_from_slice(&bytes);
        out
    }

    fn encode_u32_word(value: u32) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[28..].copy_from_slice(&value.to_be_bytes());
        out
    }

    fn keccak_word(value: &str) -> [u8; 32] {
        Keccak256::digest(value.as_bytes()).into()
    }

    fn manual_domain_separator(domain: &TypedDataDomain) -> [u8; 32] {
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&Keccak256::digest(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
                .as_bytes(),
        ));
        encoded.extend_from_slice(&keccak_word(&domain.name));
        encoded.extend_from_slice(&keccak_word(&domain.version));
        encoded.extend_from_slice(&encode_u256_word(&domain.chain_id.to_string()));
        encoded.extend_from_slice(&encode_address_word(&domain.verifying_contract));
        Keccak256::digest(&encoded).into()
    }

    fn manual_struct_hash(order: &NormalizedOrder) -> [u8; 32] {
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&hex::decode(ORDER_TYPE_HASH.trim_start_matches("0x")).unwrap());
        encoded.extend_from_slice(&encode_address_word(&order.sell_token));
        encoded.extend_from_slice(&encode_address_word(&order.buy_token));
        encoded.extend_from_slice(&encode_address_word(&order.receiver));
        encoded.extend_from_slice(&encode_u256_word(&order.sell_amount.to_string()));
        encoded.extend_from_slice(&encode_u256_word(&order.buy_amount.to_string()));
        encoded.extend_from_slice(&encode_u32_word(order.valid_to));
        encoded.extend_from_slice(
            &hex::decode(order.app_data.as_str().trim_start_matches("0x")).unwrap(),
        );
        encoded.extend_from_slice(&encode_u256_word(&order.fee_amount.to_string()));
        encoded.extend_from_slice(&keccak_word("sell"));
        encoded.extend_from_slice(&{
            let mut out = [0u8; 32];
            out[31] = 1;
            out
        });
        encoded.extend_from_slice(&keccak_word("external"));
        encoded.extend_from_slice(&keccak_word("internal"));
        Keccak256::digest(&encoded).into()
    }

    #[test]
    fn order_hash_and_struct_hash_match_manual_eip712_encoding() {
        let domain = sample_domain();
        let order = sample_order();
        let normalized = normalize_order(&order).unwrap();
        let expected_struct_hash = manual_struct_hash(&normalized);

        let mut digest_payload = Vec::with_capacity(66);
        digest_payload.extend_from_slice(&[0x19, 0x01]);
        digest_payload.extend_from_slice(&manual_domain_separator(&domain));
        digest_payload.extend_from_slice(&expected_struct_hash);
        let expected_digest = Keccak256::digest(&digest_payload);

        assert_eq!(
            order_struct_hash(&normalized).unwrap(),
            expected_struct_hash
        );
        assert_eq!(
            hash_order(&domain, &order).unwrap().as_str(),
            format!("0x{}", hex::encode(expected_digest))
        );
    }
}
