use alloy_primitives::{B256, LogData, keccak256};
use cow_sdk_core::{Address, BuyTokenDestination, OrderKind, SellTokenSource};

use crate::ContractsError;

pub(crate) const ORDER_UID_LENGTH_BYTES: usize = 56;

/// Right-aligns a cow [`Address`] into a 32-byte ABI word.
///
/// The EVM ABI lays out addresses as `bytes32` words whose low-order 20
/// bytes carry the canonical address payload and the high-order 12 bytes
/// are zero. The cow [`Address`] newtype is `#[repr(transparent)]` over
/// [`alloy_primitives::Address`] per ADR 0052, so the conversion is a
/// borrow of the inner 20-byte slice with no hex parsing or
/// reallocation. Production callers that target the ERC-20 / settlement
/// ABI surface route through this helper to keep the single canonical
/// pre-encoded shape across the workspace.
#[must_use]
pub fn encode_address_word(address: &Address) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[12..].copy_from_slice(address.as_slice());
    out
}

/// Returns the EIP-712 type-string label for a supported order kind.
///
/// The `"buy"` and `"sell"` labels feed into the keccak preimage of the
/// `GPv2Order` typed-data `kind` field. The mapping is the canonical
/// shared table that the signing, hashing, and signature-verification
/// helpers in this workspace route through so the on-chain and
/// typed-data views agree on the label bytes.
#[must_use]
pub const fn order_kind_name(kind: OrderKind) -> &'static str {
    match kind {
        OrderKind::Buy => "buy",
        OrderKind::Sell => "sell",
    }
}

/// Returns the settlement flag label for a supported sell-token balance source.
///
/// # Panics
///
/// Panics only if a new balance-source variant reaches this internal codec
/// before the settlement flag mapping is updated.
#[must_use]
pub fn sell_balance_name(balance: SellTokenSource) -> &'static str {
    match balance {
        SellTokenSource::Erc20 => "erc20",
        SellTokenSource::External => "external",
        SellTokenSource::Internal => "internal",
        // SAFETY: all currently representable settlement sell-token balance
        // variants are handled above; new variants must update this codec.
        _ => unreachable!("SellTokenSource variants are exhaustively covered"),
    }
}

/// Returns the settlement flag label for a supported buy-token balance destination.
///
/// # Panics
///
/// Panics only if a new balance-destination variant reaches this internal codec
/// before the settlement flag mapping is updated.
#[must_use]
pub fn buy_balance_name(balance: BuyTokenDestination) -> &'static str {
    match balance {
        BuyTokenDestination::Erc20 => "erc20",
        BuyTokenDestination::Internal => "internal",
        // SAFETY: all currently representable settlement buy-token balance
        // variants are handled above; new variants must update this codec.
        _ => unreachable!("BuyTokenDestination variants are exhaustively covered"),
    }
}

/// Resolves an order kind from its `GPv2` marker hash.
///
/// On the `GPv2Order` event ABI the `kind` field is a `bytes32` marker equal to
/// the keccak-256 of the order-kind label (`"sell"` / `"buy"`). This is the
/// inverse of [`order_kind_name`] and is keyed on the same canonical labels, so
/// the forward and reverse mappings cannot drift.
///
/// # Errors
///
/// Returns [`ContractsError::UnknownOrderMarker`] when `marker` does not equal
/// the keccak-256 of any supported order-kind label.
pub fn order_kind_from_marker(marker: B256) -> Result<OrderKind, ContractsError> {
    for candidate in [OrderKind::Buy, OrderKind::Sell] {
        if marker == keccak256(order_kind_name(candidate).as_bytes()) {
            return Ok(candidate);
        }
    }
    Err(ContractsError::UnknownOrderMarker(marker))
}

/// Resolves a sell-token balance source from its `GPv2` marker hash.
///
/// The `sellTokenBalance` field on the `GPv2Order` event ABI is a `bytes32`
/// marker equal to the keccak-256 of the settlement flag label
/// (`"erc20"` / `"external"` / `"internal"`). Inverse of [`sell_balance_name`].
///
/// # Errors
///
/// Returns [`ContractsError::UnknownOrderMarker`] when `marker` does not equal
/// the keccak-256 of any supported sell-token balance label.
pub fn sell_balance_from_marker(marker: B256) -> Result<SellTokenSource, ContractsError> {
    for candidate in [
        SellTokenSource::Erc20,
        SellTokenSource::External,
        SellTokenSource::Internal,
    ] {
        if marker == keccak256(sell_balance_name(candidate).as_bytes()) {
            return Ok(candidate);
        }
    }
    Err(ContractsError::UnknownOrderMarker(marker))
}

/// Resolves a buy-token balance destination from its `GPv2` marker hash.
///
/// The `buyTokenBalance` field on the `GPv2Order` event ABI is a `bytes32`
/// marker equal to the keccak-256 of the settlement flag label
/// (`"erc20"` / `"internal"`). Inverse of [`buy_balance_name`].
///
/// # Errors
///
/// Returns [`ContractsError::UnknownOrderMarker`] when `marker` does not equal
/// the keccak-256 of any supported buy-token balance label.
pub fn buy_balance_from_marker(marker: B256) -> Result<BuyTokenDestination, ContractsError> {
    for candidate in [BuyTokenDestination::Erc20, BuyTokenDestination::Internal] {
        if marker == keccak256(buy_balance_name(candidate).as_bytes()) {
            return Ok(candidate);
        }
    }
    Err(ContractsError::UnknownOrderMarker(marker))
}

/// Validates that an on-chain event log carries the expected topic-0 signature
/// hash and indexed-parameter arity before ABI decoding.
///
/// This is the shared fail-closed topic guard used by the on-chain order and
/// settlement event decoders: it rejects a malformed or hostile topic set with
/// a typed error instead of letting a later slice or index panic on untrusted
/// log bytes.
///
/// # Errors
///
/// Returns [`ContractsError::UnexpectedEventTopics`] when the topic count does
/// not equal `expected_len` or when `topics[0]` does not equal
/// `expected_topic0`.
pub(crate) fn check_topics(
    log: &LogData,
    expected_topic0: B256,
    expected_len: usize,
    event: &'static str,
) -> Result<(), ContractsError> {
    let topics = log.topics();
    if topics.len() != expected_len || topics.first() != Some(&expected_topic0) {
        return Err(ContractsError::UnexpectedEventTopics { event });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::U256;
    use alloy_sol_types::Eip712Domain;
    use std::str::FromStr;

    #[test]
    fn domain_separator_matches_shared_parity_fixture() {
        const FIXTURE: &str = include_str!("../tests/fixtures/domain_separator_parity.json");
        let fixture: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("domain separator fixture must parse");
        assert_eq!(fixture["schema_version"].as_u64(), Some(1));
        let case = &fixture["case"];
        let name = case["name"].as_str().expect("fixture case must carry name");
        let version = case["version"]
            .as_str()
            .expect("fixture case must carry version");
        let chain_id = case["chain_id"]
            .as_u64()
            .expect("fixture case must carry chain_id");
        let verifying_contract_str = case["verifying_contract"]
            .as_str()
            .expect("fixture case must carry verifying_contract");
        let verifying_contract = alloy_primitives::Address::from_str(verifying_contract_str)
            .expect("fixture verifying_contract must be a valid address");
        let expected_separator = case["domain_separator"]
            .as_str()
            .expect("fixture case must carry domain_separator");

        let domain = Eip712Domain {
            name: Some(name.to_owned().into()),
            version: Some(version.to_owned().into()),
            chain_id: Some(U256::from(chain_id)),
            verifying_contract: Some(verifying_contract),
            salt: None,
        };
        let actual = format!("{}", domain.separator());

        assert_eq!(actual, expected_separator);
    }

    #[test]
    fn order_kind_name_table_matches_protocol_labels() {
        assert_eq!(order_kind_name(OrderKind::Buy), "buy");
        assert_eq!(order_kind_name(OrderKind::Sell), "sell");
    }

    #[test]
    fn order_kind_marker_round_trips_and_rejects_unknown() {
        for kind in [OrderKind::Buy, OrderKind::Sell] {
            let marker = keccak256(order_kind_name(kind).as_bytes());
            assert_eq!(order_kind_from_marker(marker).unwrap(), kind);
        }
        assert!(matches!(
            order_kind_from_marker(B256::repeat_byte(0x01)),
            Err(ContractsError::UnknownOrderMarker(_))
        ));
    }

    #[test]
    fn sell_balance_marker_round_trips_and_rejects_unknown() {
        for source in [
            SellTokenSource::Erc20,
            SellTokenSource::External,
            SellTokenSource::Internal,
        ] {
            let marker = keccak256(sell_balance_name(source).as_bytes());
            assert_eq!(sell_balance_from_marker(marker).unwrap(), source);
        }
        assert!(matches!(
            sell_balance_from_marker(B256::repeat_byte(0x02)),
            Err(ContractsError::UnknownOrderMarker(_))
        ));
    }

    #[test]
    fn buy_balance_marker_round_trips_and_rejects_unknown() {
        for destination in [BuyTokenDestination::Erc20, BuyTokenDestination::Internal] {
            let marker = keccak256(buy_balance_name(destination).as_bytes());
            assert_eq!(buy_balance_from_marker(marker).unwrap(), destination);
        }
        assert!(matches!(
            buy_balance_from_marker(B256::repeat_byte(0x03)),
            Err(ContractsError::UnknownOrderMarker(_))
        ));
    }
}
