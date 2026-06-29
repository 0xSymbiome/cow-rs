//! Typed ABI bindings for the ERC-20 and wrapped-native token surfaces.
//!
//! - [`IERC20`] reproduces the minimal [EIP-20](https://eips.ethereum.org/EIPS/eip-20)
//!   surface every downstream consumer needs (`balanceOf`, `approve`,
//!   `allowance`, `transfer`, `transferFrom`) plus the standard `Transfer` and
//!   `Approval` events. [`encode_approve`] emits the `approve(spender, amount)`
//!   call-data and [`approve_transaction`] wraps it into a gas-free
//!   [`UnsignedTransaction`].
//! - [`IWrappedNativeToken`] reproduces the WETH9-family `deposit()` /
//!   `withdraw(uint256)` surface, with [`wrap_interaction`] /
//!   [`unwrap_interaction`] helpers that emit the canonical settlement
//!   [`Interaction`] for converting between the native asset and its wrapped
//!   form, and the [`wrap_transaction`] / [`unwrap_transaction`] builders that
//!   wrap those into a gas-free [`UnsignedTransaction`]. The wrapped-native
//!   token address for a chain is resolved by
//!   [`cow_sdk_core::wrapped_native_token`].
//!
//! Both interfaces are authored inline as `alloy::sol!` against the published
//! EIP-20 standard and the upstream `cowprotocol/ethflowcontract`
//! `src/interfaces/IWrappedNativeToken.sol` surface (pinned by commit in
//! `parity/source-lock.yaml`); their wire shape is proven by the selector
//! fixtures under `parity/fixtures/` and the crate parity tests.

use alloy_primitives::Bytes;
use alloy_sol_types::{SolCall, sol};

use cow_sdk_core::{Address, Amount, HexData, SupportedChainId, wrapped_native_token};

use crate::interaction::Interaction;
use crate::tx::UnsignedTransaction;

sol! {
    /// Minimal ERC-20 interface.
    ///
    /// Reproduces the canonical surface defined by
    /// <https://eips.ethereum.org/EIPS/eip-20>: `balanceOf`, `approve`,
    /// `allowance`, `transfer`, `transferFrom` plus the `Transfer` and
    /// `Approval` events. Every method selector and event topic matches the
    /// standard wire shape byte-identically; the 4-byte selectors generated
    /// through this binding are the same bytes emitted by every EIP-20
    /// implementation.
    #[sol(rename_all = "camelcase")]
    interface IERC20 {
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);

        function balanceOf(address account) external view returns (uint256);

        function transfer(address to, uint256 value) external returns (bool);

        function allowance(address owner, address spender) external view returns (uint256);

        function approve(address spender, uint256 value) external returns (bool);

        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }
}

sol! {
    // Canonical wrapped-native-token surface. The `deposit()` / `withdraw(uint256)`
    // signatures mirror cowprotocol/ethflowcontract
    // `src/interfaces/IWrappedNativeToken.sol` (pinned by commit in
    // `parity/source-lock.yaml`); the 4-byte selectors they generate are
    // byte-identical to those every WETH9 deployment exposes and are proven by
    // the selector fixtures under `parity/fixtures/`.
    interface IWrappedNativeToken {
        function deposit() external payable;
        function withdraw(uint256 wad) external;
    }
}

/// Builds the interaction that wraps `amount` of the native asset into the
/// wrapped-native token.
///
/// The interaction calls `deposit()` on `wrapped_native_token` with `amount`
/// attached as the native value transferred with the call.
#[must_use]
pub fn wrap_interaction(wrapped_native_token: Address, amount: Amount) -> Interaction {
    Interaction::new(
        wrapped_native_token,
        amount,
        Bytes::from(IWrappedNativeToken::depositCall {}.abi_encode()),
    )
}

/// Builds the interaction that unwraps `amount` of the wrapped-native token back
/// into the native asset.
///
/// The interaction calls `withdraw(amount)` on `wrapped_native_token` with zero
/// native value attached.
#[must_use]
pub fn unwrap_interaction(wrapped_native_token: Address, amount: Amount) -> Interaction {
    Interaction::new(
        wrapped_native_token,
        Amount::ZERO,
        Bytes::from(
            IWrappedNativeToken::withdrawCall {
                wad: *amount.as_u256(),
            }
            .abi_encode(),
        ),
    )
}

/// Builds the gas-free transaction that wraps `amount` of the chain's native
/// asset into its wrapped-native token (for example ETH into WETH).
///
/// The target is the chain's canonical wrapped-native token, resolved from
/// `chain_id`; `amount` is sent as the call's native value. The returned
/// [`UnsignedTransaction`] carries no gas limit; the caller estimates gas,
/// signs, and submits.
#[must_use]
pub fn wrap_transaction(chain_id: SupportedChainId, amount: Amount) -> UnsignedTransaction {
    wrap_interaction(wrapped_native_token(chain_id).address, amount).into()
}

/// Builds the gas-free transaction that unwraps `amount` of the wrapped-native
/// token back into the chain's native asset (for example WETH into ETH).
///
/// The target is the chain's canonical wrapped-native token, resolved from
/// `chain_id`. `withdraw` burns the caller's own wrapped-native balance, so no
/// ERC-20 approval is required and no native value is attached. The returned
/// [`UnsignedTransaction`] carries no gas limit; the caller estimates gas,
/// signs, and submits.
#[must_use]
pub fn unwrap_transaction(chain_id: SupportedChainId, amount: Amount) -> UnsignedTransaction {
    unwrap_interaction(wrapped_native_token(chain_id).address, amount).into()
}

/// Returns the ABI-encoded `approve(spender, amount)` call-data for an ERC-20
/// token, granting `spender` an allowance of `amount`.
///
/// Infallible: the cow [`Amount`] newtype enforces the `uint256` boundary at
/// construction per ADR 0052, so the alloy-sol `abi_encode` call cannot fail by
/// construction.
#[must_use]
pub fn encode_approve(spender: &Address, amount: &Amount) -> Vec<u8> {
    IERC20::approveCall {
        spender: (*spender).into(),
        value: *amount.as_u256(),
    }
    .abi_encode()
}

/// Builds the gas-free ERC-20 approval transaction that grants `spender` an
/// allowance of `amount` on `token`.
///
/// The target is `token`, the value is zero, and the call-data is the
/// [`encode_approve`] payload. The `CoW` Protocol vault relayer is the usual
/// `spender`; the caller resolves its deployment through
/// [`resolve_contract_address`](crate::resolve_contract_address) and passes it
/// here. The returned [`UnsignedTransaction`] carries no gas limit; the caller
/// estimates gas, signs, and submits.
#[must_use]
pub fn approve_transaction(
    token: Address,
    spender: Address,
    amount: Amount,
) -> UnsignedTransaction {
    UnsignedTransaction::new(
        token,
        HexData::from_bytes(encode_approve(&spender, &amount)),
        Amount::ZERO,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        IWrappedNativeToken, approve_transaction, encode_approve, unwrap_transaction,
        wrap_transaction,
    };
    use alloy_sol_types::SolCall as _;
    use cow_sdk_core::{Address, Amount, HexData, SupportedChainId, wrapped_native_token};

    #[test]
    fn wrap_transaction_resolves_canonical_address_and_sends_amount_as_value() {
        let amount = Amount::from(1_000u32);
        let tx = wrap_transaction(SupportedChainId::Mainnet, amount);

        assert_eq!(
            tx.to,
            wrapped_native_token(SupportedChainId::Mainnet).address
        );
        assert_eq!(tx.value, amount);
        assert_eq!(
            tx.data,
            HexData::from_bytes(IWrappedNativeToken::depositCall {}.abi_encode())
        );
    }

    #[test]
    fn unwrap_transaction_resolves_canonical_address_with_zero_value() {
        let amount = Amount::from(1_000u32);
        let tx = unwrap_transaction(SupportedChainId::Base, amount);

        assert_eq!(tx.to, wrapped_native_token(SupportedChainId::Base).address);
        assert_eq!(tx.value, Amount::ZERO);
        assert_eq!(
            tx.data,
            HexData::from_bytes(
                IWrappedNativeToken::withdrawCall {
                    wad: *amount.as_u256(),
                }
                .abi_encode()
            )
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

    #[test]
    fn approve_transaction_targets_token_with_zero_value_and_encodes_spender() {
        let token = Address::from_bytes([0x11; 20]);
        let spender = Address::from_bytes([0x22; 20]);
        let amount = Amount::from(5_000u32);
        let tx = approve_transaction(token, spender, amount);

        assert_eq!(tx.to, token);
        assert_eq!(tx.value, Amount::ZERO);
        assert_eq!(
            tx.data,
            HexData::from_bytes(encode_approve(&spender, &amount))
        );
    }
}
