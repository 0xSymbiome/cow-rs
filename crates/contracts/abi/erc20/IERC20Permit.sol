// SPDX-License-Identifier: MIT
pragma solidity >=0.7.6 <0.9.0;

// Provenance
// ----------
// The EIP-2612 `permit` surface reproduced below is the standard defined by
// EIP-2612 (https://eips.ethereum.org/EIPS/eip-2612). The canonical Solidity
// reference implementation lives in the OpenZeppelin contracts library at
// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/
// contracts/token/ERC20/extensions/IERC20Permit.sol, and the EIP-712
// typed-data hashing rules for the `Permit` struct are set by EIP-712
// (https://eips.ethereum.org/EIPS/eip-712).
//
// The `Permit` struct-hash preimage is the canonical 5-field form
//   "Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)"
// which yields the type hash
//   0x6e71edae12b1b97f4d1f60370fef10105fa2faae0126114a169c64845d6126c9
// used across every EIP-2612 deployment. The `nonce` field appears in the
// struct-hash preimage but is NOT a call argument on `permit(...)`: the
// token contract reads the current nonce from storage during the call, so
// the caller only supplies `(owner, spender, value, deadline, v, r, s)`.
//
// This file is documentation-only: it preserves upstream provenance for
// reviewers. The Rust bindings derived from the same signatures live in
// `crates/contracts/src/erc20.rs`.

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
