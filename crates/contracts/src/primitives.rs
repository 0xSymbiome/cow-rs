use alloy_primitives::keccak256;

use cow_sdk_core::{BuyTokenDestination, OrderKind, SellTokenSource};

pub(crate) const ORDER_UID_LENGTH_BYTES: usize = 56;

pub(crate) const fn order_kind_name(kind: OrderKind) -> &'static str {
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
pub(crate) fn sell_balance_name(balance: SellTokenSource) -> &'static str {
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
pub(crate) fn buy_balance_name(balance: BuyTokenDestination) -> &'static str {
    match balance {
        BuyTokenDestination::Erc20 => "erc20",
        BuyTokenDestination::Internal => "internal",
        // SAFETY: all currently representable settlement buy-token balance
        // variants are handled above; new variants must update this codec.
        _ => unreachable!("BuyTokenDestination variants are exhaustively covered"),
    }
}

pub(crate) fn function_selector(signature: &str) -> [u8; 4] {
    let hash = keccak256(signature.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]]
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
}
