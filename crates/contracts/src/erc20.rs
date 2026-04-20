//! Typed ABI bindings for ERC-20 and EIP-2612 Permit.
//!
//! The module exposes two `alloy::sol!`-generated interfaces:
//!
//! * [`IERC20`] — the minimal [EIP-20](https://eips.ethereum.org/EIPS/eip-20)
//!   surface every downstream consumer in this workspace needs (`balanceOf`,
//!   `approve`, `allowance`, `transfer`, `transferFrom`) plus the standard
//!   `Transfer` and `Approval` events.
//!
//! * [`IERC20Permit`] — the
//!   [EIP-2612](https://eips.ethereum.org/EIPS/eip-2612) typed-data `permit`
//!   extension, together with the `Permit` struct used to derive the
//!   [EIP-712](https://eips.ethereum.org/EIPS/eip-712) typed-data hash.
//!
//! The `Permit` struct uses the canonical 5-field preimage
//! `"Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)"`
//! — the `nonce` field appears in the struct-hash preimage but is not a
//! call argument on `permit(...)`: the token contract reads the current
//! nonce from storage during the call, so callers supply
//! `(owner, spender, value, deadline, v, r, s)` only.
//!
//! [`PERMIT_TYPE_HASH`] pins the canonical EIP-2612 type hash, and
//! [`permit_typed_data_hash`] composes the EIP-712 domain separator with the
//! struct hash so off-chain signers can produce a 32-byte digest that any
//! EIP-2612 token contract will accept.
//!
//! The Solidity excerpts used to author these bindings are committed under
//! `crates/contracts/abi/erc20/` for provenance.

use alloy_sol_types::{Eip712Domain, SolStruct, sol};

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

    /// EIP-2612 Permit extension.
    ///
    /// `permit` authorizes an allowance through a typed-data signature rather
    /// than an on-chain approval transaction. The typed-data hash combines
    /// the token's EIP-712 domain separator (parameterized by `name`,
    /// `version`, `chainId`, and `verifyingContract`) with the keccak-256 of
    /// the canonical 5-field `Permit` struct:
    ///
    /// ```text
    /// Permit(
    ///   address owner,
    ///   address spender,
    ///   uint256 value,
    ///   uint256 nonce,
    ///   uint256 deadline
    /// )
    /// ```
    ///
    /// The `nonce` field is part of the struct-hash preimage but is not a
    /// call argument on `permit(...)`; the contract reads it from storage
    /// during the call. Off-chain signers can build the 32-byte typed-data
    /// digest through [`permit_typed_data_hash`].
    #[sol(rename_all = "camelcase")]
    interface IERC20Permit {
        struct Permit {
            address owner;
            address spender;
            uint256 value;
            uint256 nonce;
            uint256 deadline;
        }

        function permit(
            address owner,
            address spender,
            uint256 value,
            uint256 deadline,
            uint8 v,
            bytes32 r,
            bytes32 s
        ) external;

        function nonces(address owner) external view returns (uint256);

        function DOMAIN_SEPARATOR() external view returns (bytes32);
    }
}

/// Canonical EIP-2612 Permit type hash.
///
/// Equals
/// `keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)")`
/// and matches the value used by every EIP-2612 deployment on the network
/// (0x6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c9).
pub const PERMIT_TYPE_HASH: [u8; 32] = [
    0x6e, 0x71, 0xed, 0xae, 0x12, 0xb1, 0xb9, 0x7f, 0x4d, 0x1f, 0x60, 0x37, 0x0f, 0xef, 0x10, 0x10,
    0x5f, 0xa2, 0xfa, 0xae, 0x01, 0x26, 0x11, 0x4a, 0x16, 0x9c, 0x64, 0x84, 0x5d, 0x61, 0x26, 0xc9,
];

/// Returns the EIP-712 typed-data hash for a `Permit` struct bound to the
/// supplied domain.
///
/// The returned 32-byte digest is the value an off-chain signer signs to
/// authorize an EIP-2612 allowance transfer: it composes the domain
/// separator derived from `domain` (token name, version, chain id, and
/// verifying contract) with the `Permit` struct hash, following the
/// `\x19\x01 || domainSeparator || structHash` envelope specified by
/// [EIP-712](https://eips.ethereum.org/EIPS/eip-712).
#[must_use]
pub fn permit_typed_data_hash(domain: &Eip712Domain, permit: &IERC20Permit::Permit) -> [u8; 32] {
    permit.eip712_signing_hash(domain).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha3::{Digest, Keccak256};

    #[test]
    fn permit_type_hash_matches_the_canonical_keccak_preimage() {
        let preimage =
            "Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)";
        let digest: [u8; 32] = Keccak256::digest(preimage.as_bytes()).into();
        assert_eq!(
            PERMIT_TYPE_HASH, digest,
            "PERMIT_TYPE_HASH must equal keccak256 of the canonical 5-field preimage",
        );
    }

    #[test]
    fn permit_sol_struct_type_hash_equals_the_exported_constant() {
        use alloy_sol_types::private::{Address, U256};

        let permit = IERC20Permit::Permit {
            owner: Address::ZERO,
            spender: Address::ZERO,
            value: U256::ZERO,
            nonce: U256::ZERO,
            deadline: U256::ZERO,
        };
        let sol_type_hash: [u8; 32] = permit.eip712_type_hash().into();
        assert_eq!(
            sol_type_hash, PERMIT_TYPE_HASH,
            "the sol!-generated Permit type hash must match the exported constant",
        );
    }
}
