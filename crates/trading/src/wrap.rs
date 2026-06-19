//! Native-asset wrapping transactions.
//!
//! Wrapping converts a chain's native asset to its wrapped-native token and back
//! (for example ETH to WETH). These helpers build the canonical `deposit()` /
//! `withdraw(uint256)` transaction for a chain, resolving the wrapped-native
//! address from the chain id through [`cow_sdk_core::wrapped_native_token`] so
//! callers never supply a token address.
//!
//! Selling native currency through `CoW` Protocol does not require a manual wrap:
//! the eth-flow path wraps on-chain as part of order creation
//! ([`eth_flow_transaction`](crate::eth_flow_transaction)). These helpers serve
//! the standalone wrap and treasury paths — holding the wrapped form to trade it
//! directly, or converting back to the native asset.
//!
//! Both helpers are infallible: the wrapped-native address is a total function of
//! the typed [`SupportedChainId`], `amount` is a construction-validated
//! [`Amount`](cow_sdk_core::Amount), and the `deposit()` / `withdraw(uint256)`
//! calldata is a fixed encoding. The calldata bytes are pinned by the selector
//! fixtures behind [`wrap_interaction`](cow_sdk_contracts::wrap_interaction) and
//! [`unwrap_interaction`](cow_sdk_contracts::unwrap_interaction).

use cow_sdk_contracts::{unwrap_interaction, wrap_interaction};
use cow_sdk_core::{Amount, SupportedChainId, TransactionRequest, wrapped_native_token};

/// Builds the transaction that wraps `amount` of the chain's native asset into
/// its wrapped-native token (for example ETH into WETH).
///
/// The target is the chain's canonical wrapped-native token, resolved from
/// `chain_id`; `amount` is sent as the call's native value. Submit the returned
/// request with any [`cow_sdk_core::Signer`].
#[must_use]
pub fn wrap_transaction(chain_id: SupportedChainId, amount: Amount) -> TransactionRequest {
    wrap_interaction(wrapped_native_token(chain_id).address, amount).into()
}

/// Builds the transaction that unwraps `amount` of the wrapped-native token back
/// into the chain's native asset (for example WETH into ETH).
///
/// The target is the chain's canonical wrapped-native token, resolved from
/// `chain_id`. `withdraw` burns the caller's own wrapped-native balance, so no
/// ERC-20 approval is required and no native value is attached.
#[must_use]
pub fn unwrap_transaction(chain_id: SupportedChainId, amount: Amount) -> TransactionRequest {
    unwrap_interaction(wrapped_native_token(chain_id).address, amount).into()
}

#[cfg(test)]
mod tests {
    use super::{unwrap_transaction, wrap_transaction};
    use cow_sdk_core::{Amount, SupportedChainId, wrapped_native_token};

    #[test]
    fn wrap_transaction_resolves_canonical_address_and_sends_amount_as_value() {
        let amount = Amount::from(1_000u32);
        let tx = wrap_transaction(SupportedChainId::Mainnet, amount);

        assert_eq!(
            tx.to,
            Some(wrapped_native_token(SupportedChainId::Mainnet).address)
        );
        assert_eq!(tx.value, Some(amount));
        assert!(
            tx.data.is_some(),
            "wrap transaction carries deposit calldata"
        );
    }

    #[test]
    fn unwrap_transaction_resolves_canonical_address_with_zero_value() {
        let amount = Amount::from(1_000u32);
        let tx = unwrap_transaction(SupportedChainId::Base, amount);

        assert_eq!(
            tx.to,
            Some(wrapped_native_token(SupportedChainId::Base).address)
        );
        assert_eq!(tx.value, Some(Amount::ZERO));
        assert!(
            tx.data.is_some(),
            "unwrap transaction carries withdraw calldata"
        );
    }

    #[test]
    fn wrap_transaction_resolves_a_distinct_address_per_chain() {
        let amount = Amount::from(1u32);
        assert_ne!(
            wrap_transaction(SupportedChainId::Mainnet, amount).to,
            wrap_transaction(SupportedChainId::GnosisChain, amount).to,
            "each chain wraps into its own native token"
        );
    }
}
